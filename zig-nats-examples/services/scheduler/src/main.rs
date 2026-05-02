mod models;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
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

    // Connect to NATS
    let nats_client = async_nats::connect(&settings.nats.url).await?;
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

    // Connect to MongoDB
    let mongo_config = settings.mongodb.as_ref().ok_or_else(|| anyhow::anyhow!("MongoDB configuration is missing"))?;
    let mongo_client = mongodb::Client::with_uri_str(&mongo_config.url).await?;
    let db = mongo_client.database(&mongo_config.db_name);
    tracing::info!("Connected to MongoDB");

    let state = AppState {
        db: db.clone(),
        nats: nats_client,
        js: js.clone(),
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

async fn health_check() -> &'static str {
    "OK"
}

async fn create_action(
    State(state): State<AppState>,
    user: JwtUser,
    Json(payload): Json<CreateActionRequest>,
) -> Result<Json<Action>, (StatusCode, String)> {
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

async fn list_actions(
    State(state): State<AppState>,
    user: JwtUser,
) -> Result<Json<Vec<Action>>, (StatusCode, String)> {
    let actions_collection = state.db.collection::<Action>("actions");

    let user_id = ObjectId::from_str(&user.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;

    let mut cursor = actions_collection
        .find(doc! { "user_id": user_id })
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
        .find_one(doc! { "_id": aid, "user_id": user_id })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Action not found".to_string()))?;

    let execution = Execution {
        id: None,
        action_id: action.id.unwrap(),
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

async fn list_executions(
    State(state): State<AppState>,
    user: JwtUser,
) -> Result<Json<Vec<Execution>>, (StatusCode, String)> {
    let executions_collection = state.db.collection::<Execution>("executions");

    let user_id = ObjectId::from_str(&user.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;

    let mut cursor = executions_collection
        .find(doc! { "user_id": user_id })
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
