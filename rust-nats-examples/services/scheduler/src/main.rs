mod models;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put, delete},
    Json, Router,
};

use std::net::SocketAddr;
use std::str::FromStr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use config::AppConfig;
use futures::StreamExt;
use mongodb::bson::{doc, oid::ObjectId, to_bson};
use shared_auth::{HasJwtSecret, JwtUser};
use models::*;
use cron::Schedule;
use chrono::Utc;

#[derive(Clone)]
struct AppState {
    db: mongodb::Database,
    nats: async_nats::Client,
    js: async_nats::jetstream::Context,
    kv: async_nats::jetstream::kv::Store,
    config: AppConfig,
}

impl HasJwtSecret for AppState {
    fn jwt_secret(&self) -> &str {
        self.config.auth.as_ref().map(|a| a.jwt_secret.as_str()).unwrap_or("")
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "scheduler=debug,info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Scheduler Service...");

    // Load configuration
    let settings = AppConfig::load().map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;
    tracing::info!("Configuration loaded");

    // Connect to NATS with retry
    let nats_client = {
        let mut retry_count = 0;
        loop {
            match async_nats::connect(&settings.nats.url).await {
                Ok(client) => break client,
                Err(e) if retry_count < 10 => {
                    retry_count += 1;
                    tracing::warn!("Failed to connect to NATS, retrying ({}/10): {}", retry_count, e);
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
                Err(e) => return Err(anyhow::anyhow!("Final NATS connection failure: {}", e)),
            }
        }
    };

    let js = async_nats::jetstream::new(nats_client.clone());
    
    // Ensure JetStream Streams exist
    js.get_or_create_stream(async_nats::jetstream::stream::Config {
        name: "ACTIONS_DISPATCH".to_string(),
        subjects: vec!["action.execute.>".to_string()],
        ..Default::default()
    }).await?;
    
    js.get_or_create_stream(async_nats::jetstream::stream::Config {
        name: "ACTIONS_OUTCOME".to_string(),
        subjects: vec!["action.outcome.>".to_string()],
        ..Default::default()
    }).await?;

    tracing::info!("Connected to NATS and JetStream initialized");

    // Connect to MongoDB with retry
    let mongo_config = settings.mongodb.as_ref().ok_or_else(|| anyhow::anyhow!("MongoDB configuration is missing"))?;
    let mongo_client = {
        let mut retry_count = 0;
        loop {
            match mongodb::Client::with_uri_str(&mongo_config.url).await {
                Ok(client) => {
                    if let Ok(_) = client.database("admin").run_command(doc! {"ping": 1}).await {
                        break client;
                    }
                }
                _ => {}
            }
            if retry_count >= 10 {
                return Err(anyhow::anyhow!("Final MongoDB connection failure"));
            }
            retry_count += 1;
            tracing::warn!("Waiting for MongoDB, retrying ({}/10)...", retry_count);
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    };
    let db = mongo_client.database(&mongo_config.db_name);
    tracing::info!("Connected to MongoDB");

    let state = AppState {
        db: db.clone(),
        nats: nats_client,
        js: js.clone(),
        kv: js.create_key_value(async_nats::jetstream::kv::Config {
            bucket: "ORG_ROLES".to_string(),
            history: 1,
            ..Default::default()
        }).await?,
        config: settings.clone(),
    };

    // Background Worker: Cron Evaluator
    let cron_state = state.clone();
    tokio::spawn(async move {
        tracing::info!("Cron Evaluator started");
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Err(e) = evaluate_crons(&cron_state).await {
                tracing::error!("Error in cron evaluator: {}", e);
            }
        }
    });

    // Background Worker: Outcome Listener
    let outcome_state = state.clone();
    tokio::spawn(async move {
        tracing::info!("Outcome Listener started");
        if let Err(e) = listen_for_outcomes(&outcome_state).await {
            tracing::error!("Error in outcome listener: {}", e);
        }
    });

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/actions", post(create_action).get(list_actions))
        .route("/actions/:id", delete(delete_action).put(update_action))
        .route("/actions/:id/trigger", post(trigger_action))
        .route("/executions", get(list_executions))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    // Run it
    let addr: SocketAddr = format!("{}:{}", settings.server.host, settings.server.port).parse()?;
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn check_permission(state: &AppState, org_id: &str, user_id: &str, allowed_roles: &[&str]) -> Result<(), (StatusCode, String)> {
    let kv_key = format!("org:{}:user:{}", org_id, user_id);
    
    // 1. Try Cache First
    if let Ok(Some(entry)) = state.kv.get(&kv_key).await {
        let cached_role = String::from_utf8_lossy(&entry);
        if allowed_roles.contains(&cached_role.as_ref()) {
            return Ok(());
        } else {
            return Err((StatusCode::FORBIDDEN, format!("User role '{}' is not authorized for this action", cached_role)));
        }
    }

    // 2. Cache Miss: Fall back to HTTP API
    let org_url = std::env::var("ORG_SERVICE_URL").unwrap_or_else(|_| "http://org:3001".to_string());
    let reqwest_client = reqwest::Client::new();
    let url = format!("{}/internal/orgs/{}/members/{}/role", org_url, org_id, user_id);
    
    let res = reqwest_client.get(&url)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to verify permissions: {}", e)))?;

    if !res.status().is_success() {
        return Err((StatusCode::FORBIDDEN, "User does not have access to this organization".to_string()));
    }

    let user_role = res.text().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // 3. Populate Cache
    let _ = state.kv.put(&kv_key, user_role.clone().into()).await;

    if allowed_roles.contains(&user_role.as_str()) {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, format!("User role '{}' is not authorized for this action", user_role)))
    }
}

async fn health_check() -> &'static str {
    "OK"
}

async fn create_action(
    State(state): State<AppState>,
    user: JwtUser,
    Json(payload): Json<CreateActionRequest>,
) -> Result<Json<Action>, (StatusCode, String)> {
    check_permission(&state, &payload.org_id, &user.user_id, &["Owner", "Admin", "Editor"]).await?;

    let actions_collection = state.db.collection::<Action>("actions");

    let user_id = ObjectId::from_str(&user.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;
    let org_id = ObjectId::from_str(&payload.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid organization ID".to_string()))?;

    // Validate cron expression if provided
    if let Some(ref cron) = payload.cron_expression {
        Schedule::from_str(cron).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid cron expression".to_string()))?;
    }

    let new_action = Action {
        id: None,
        user_id,
        org_id,
        action_type: payload.action_type,
        payload: payload.payload,
        trigger_type: payload.trigger_type,
        cron_expression: payload.cron_expression,
        created_at: Utc::now(),
    };

    let result = actions_collection
        .insert_one(new_action.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut action = new_action;
    action.id = Some(result.inserted_id.as_object_id().unwrap());

    Ok(Json(action))
}

use serde::Deserialize;
#[derive(Deserialize)]
pub struct ListQuery {
    pub org_id: Option<String>,
}

async fn list_actions(
    State(state): State<AppState>,
    user: JwtUser,
    axum::extract::Query(query): axum::extract::Query<ListQuery>,
) -> Result<Json<Vec<Action>>, (StatusCode, String)> {
    let actions_collection = state.db.collection::<Action>("actions");

    let mut filter = doc! {};
    if let Some(org_id_str) = query.org_id {
        // Enforce RBAC for the organization
        check_permission(&state, &org_id_str, &user.user_id, &["Owner", "Admin", "Editor", "Invoker", "Viewer"]).await?;
        if let Ok(org_oid) = ObjectId::from_str(&org_id_str) {
            filter.insert("org_id", org_oid);
        }
    } else {
        // Fallback to only actions created by this user if no org selected
        let user_id = ObjectId::from_str(&user.user_id)
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;
        filter.insert("user_id", user_id);
    }

    let mut cursor = actions_collection
        .find(filter)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut actions = Vec::new();
    while let Some(action) = cursor.next().await {
        if let Ok(action) = action {
            actions.push(action);
        }
    }

    Ok(Json(actions))
}

async fn delete_action(
    State(state): State<AppState>,
    user: JwtUser,
    Path(action_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let actions_collection = state.db.collection::<Action>("actions");
    let user_id = ObjectId::from_str(&user.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;
    let aid = ObjectId::from_str(&action_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid action ID".to_string()))?;

    let result = actions_collection
        .delete_one(doc! { "_id": aid, "user_id": user_id })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.deleted_count == 0 {
        return Err((StatusCode::NOT_FOUND, "Action not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn update_action(
    State(state): State<AppState>,
    user: JwtUser,
    Path(action_id): Path<String>,
    Json(payload): Json<UpdateActionRequest>,
) -> Result<Json<Action>, (StatusCode, String)> {
    let actions_collection = state.db.collection::<Action>("actions");
    let user_id = ObjectId::from_str(&user.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;
    let aid = ObjectId::from_str(&action_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid action ID".to_string()))?;

    let action = actions_collection
        .find_one(doc! { "_id": aid })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Action not found".to_string()))?;

    check_permission(&state, &action.org_id.to_hex(), &user.user_id, &["Owner", "Admin", "Editor"]).await?;

    // Validate cron expression if provided
    if let Some(ref cron) = payload.cron_expression {
        Schedule::from_str(cron).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid cron expression".to_string()))?;
    }

    let mut update_doc = doc! {};
    if let Some(action_type) = payload.action_type {
        update_doc.insert("action_type", action_type);
    }
    if let Some(p) = payload.payload {
        update_doc.insert("payload", to_bson(&p).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?);
    }
    if let Some(trigger_type) = payload.trigger_type {
        let tt_str = serde_json::to_string(&trigger_type).unwrap().replace("\"", "");
        update_doc.insert("trigger_type", tt_str);
    }
    if let Some(cron_expression) = payload.cron_expression {
        update_doc.insert("cron_expression", cron_expression);
    }

    if update_doc.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No fields to update".to_string()));
    }

    let result = actions_collection
        .find_one_and_update(
            doc! { "_id": aid, "user_id": user_id },
            doc! { "$set": update_doc }
        )
        .return_document(mongodb::options::ReturnDocument::After)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match result {
        Some(action) => Ok(Json(action)),
        None => Err((StatusCode::NOT_FOUND, "Action not found".to_string())),
    }
}

async fn trigger_action(
    State(state): State<AppState>,
    user: JwtUser,
    Path(action_id): Path<String>,
) -> Result<Json<Execution>, (StatusCode, String)> {
    let actions_collection = state.db.collection::<Action>("actions");
    let executions_collection = state.db.collection::<Execution>("executions");

    let user_id = ObjectId::from_str(&user.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;
    let aid = ObjectId::from_str(&action_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid action ID".to_string()))?;

    let action = actions_collection
        .find_one(doc! { "_id": aid })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Action not found".to_string()))?;

    check_permission(&state, &action.org_id.to_hex(), &user.user_id, &["Owner", "Admin", "Editor", "Invoker"]).await?;

    let execution = Execution {
        id: None,
        action_id: action.id.unwrap(),
        org_id: action.org_id,
        user_id,
        status: ExecutionStatus::Pending,
        started_at: Utc::now(),
        completed_at: None,
        result: None,
    };

    let result = executions_collection
        .insert_one(execution.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut execution = execution;
    let eid = result.inserted_id.as_object_id().unwrap();
    execution.id = Some(eid);

    // Dispatch to NATS
    let subject = format!("action.execute.{}", action.action_type);
    let payload = serde_json::json!({
        "execution_id": eid.to_hex(),
        "action_id": action.id.unwrap().to_hex(),
        "payload": action.payload
    });
    
    let payload_bytes = serde_json::to_vec(&payload)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Serialization failed: {}", e)))?;
    
    state.js.publish(subject, payload_bytes.into()).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to dispatch action: {}", e)))?;

    Ok(Json(execution))
}

#[derive(Deserialize)]
pub struct ExecQuery {
    pub action_id: Option<String>,
    pub org_id: Option<String>,
}

async fn list_executions(
    State(state): State<AppState>,
    user: JwtUser,
    axum::extract::Query(query): axum::extract::Query<ExecQuery>,
) -> Result<Json<Vec<Execution>>, (StatusCode, String)> {
    let executions_collection = state.db.collection::<Execution>("executions");

    let mut filter = doc! {};
    if let Some(act_id_str) = query.action_id {
        if let Ok(act_oid) = ObjectId::from_str(&act_id_str) {
            filter.insert("action_id", act_oid);
        }
    }
    
    if let Some(org_id_str) = query.org_id {
        check_permission(&state, &org_id_str, &user.user_id, &["Owner", "Admin", "Editor", "Invoker", "Viewer"]).await?;
        if let Ok(org_oid) = ObjectId::from_str(&org_id_str) {
            filter.insert("org_id", org_oid);
        }
    } else if filter.is_empty() {
        let user_id = ObjectId::from_str(&user.user_id)
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;
        filter.insert("user_id", user_id);
    }

    let mut cursor = executions_collection
        .find(filter)
        .sort(doc! { "started_at": -1 })
        .limit(50)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut executions = Vec::new();
    while let Some(execution) = cursor.next().await {
        if let Ok(execution) = execution {
            executions.push(execution);
        }
    }

    Ok(Json(executions))
}

async fn evaluate_crons(state: &AppState) -> anyhow::Result<()> {
    let actions_collection = state.db.collection::<Action>("actions");
    let executions_collection = state.db.collection::<Execution>("executions");

    let mut cursor = actions_collection.find(doc! { "trigger_type": "Cron" }).await?;
    
    while let Some(Ok(action)) = cursor.next().await {
        if let Some(ref cron_str) = action.cron_expression {
            let schedule = Schedule::from_str(cron_str)?;
            let now = Utc::now();
            
            if let Some(next) = schedule.upcoming(Utc).next() {
                if (next - now).num_seconds().abs() < 30 {
                    tracing::info!("Triggering cron action: {:?}", action.id);
                    
                    let execution = Execution {
                        id: None,
                        action_id: action.id.unwrap(),
                        org_id: action.org_id,
                        user_id: action.user_id,
                        status: ExecutionStatus::Pending,
                        started_at: Utc::now(),
                        completed_at: None,
                        result: None,
                    };

                    let result = executions_collection.insert_one(execution).await?;
                    let eid = result.inserted_id.as_object_id().unwrap();

                    let subject = format!("action.execute.{}", action.action_type);
                    let payload = serde_json::json!({
                        "execution_id": eid.to_hex(),
                        "action_id": action.id.unwrap().to_hex(),
                        "payload": action.payload
                    });
                    
                    state.js.publish(subject, serde_json::to_vec(&payload)?.into()).await?;
                }
            }
        }
    }
    
    Ok(())
}

async fn listen_for_outcomes(state: &AppState) -> anyhow::Result<()> {
    let stream = state.js.get_stream("ACTIONS_OUTCOME").await?;
    let consumer = stream.get_or_create_consumer("scheduler-outcome-listener", async_nats::jetstream::consumer::pull::Config {
        durable_name: Some("scheduler-outcome-listener".to_string()),
        ..Default::default()
    }).await?;

    let mut messages = consumer.messages().await?;
    let executions_collection = state.db.collection::<Execution>("executions");

    while let Some(Ok(message)) = messages.next().await {
        let outcome: serde_json::Value = serde_json::from_slice(&message.payload)?;
        
        if let Some(eid_str) = outcome["execution_id"].as_str() {
            if let Ok(eid) = ObjectId::from_str(eid_str) {
                let status = match outcome["status"].as_str() {
                    Some("Completed") => ExecutionStatus::Completed,
                    _ => ExecutionStatus::Failed,
                };
                
                let bson_result = to_bson(&outcome["result"])?;
                
                executions_collection.update_one(
                    doc! { "_id": eid },
                    doc! {
                        "$set": {
                            "status": serde_json::to_string(&status)?.replace("\"", ""),
                            "completed_at": Utc::now().to_rfc3339(),
                            "result": bson_result
                        }
                    }
                ).await?;
                
                tracing::info!("Execution {} marked as {:?}", eid_str, status);
            }
        }
        
        message.ack().await.map_err(|e| anyhow::anyhow!("Ack failed: {}", e))?;
    }
    
    Ok(())
}
