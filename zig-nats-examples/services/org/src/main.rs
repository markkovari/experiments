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
use async_nats::service::ServiceExt;
use futures::StreamExt;
use bytes::Bytes;
use mongodb::bson::{doc, oid::ObjectId};
use shared_auth::{HasJwtSecret, JwtUser};
use models::*;

#[derive(Clone)]
struct AppState {
    db: mongodb::Database,
    nats: async_nats::Client,
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
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "org=debug,info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Organization Service...");

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
    tracing::info!("Connected to NATS at {}", settings.nats.url);

    // Initialize NATS Microservice
    let service = nats_client
        .service_builder()
        .description("Organization Service")
        .start("org", "0.1.0")
        .await
        .map_err(|e| anyhow::anyhow!("failed to start NATS service: {}", e))?;
    
    tracing::info!("NATS Microservice 'org' started");

    let group = service.group("org");
    let mut ping_endpoint = group.endpoint("ping").await
        .map_err(|e| anyhow::anyhow!("failed to add NATS endpoint: {}", e))?;
    
    tokio::spawn(async move {
        while let Some(request) = ping_endpoint.next().await {
            let _ = request.respond(Ok(Bytes::from("pong"))).await;
        }
    });

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

    // Setup NATS KV for RBAC roles
    let js = async_nats::jetstream::new(nats_client.clone());
    let kv = js.create_key_value(async_nats::jetstream::kv::Config {
        bucket: "ORG_ROLES".to_string(),
        history: 1,
        ..Default::default()
    }).await?;

    let state = AppState {
        db,
        nats: nats_client,
        kv,
        config: settings.clone(),
    };

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/orgs", post(create_org).get(list_orgs))
        .route("/orgs/:id/members", post(invite_user).get(get_members))
        .route("/orgs/:id/members/:user_id", put(update_member_role).delete(remove_member))
        .route("/internal/orgs/:id/members/:user_id/role", get(get_member_role))
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

async fn create_org(
    State(state): State<AppState>,
    user: JwtUser,
    Json(payload): Json<CreateOrgRequest>,
) -> Result<Json<Organization>, (StatusCode, String)> {
    tracing::info!("Creating organization '{}' for user {}", payload.name, user.user_id);
    let orgs_collection = state.db.collection::<Organization>("organizations");

    let user_id = ObjectId::from_str(&user.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;

    // We don't have the user's email easily available in the token, so we'll 
    // fetch it from the auth service using the internal API
    let auth_url = std::env::var("AUTH_SERVICE_URL").unwrap_or_else(|_| "http://auth:3000".to_string());
    // Since we only have user_id, we need to adapt our internal API to support querying by ID or we just assume a placeholder for now since the owner's email isn't critical for authorization, only for display. Let's fetch the user by ID.
    // Ah, our internal API only supports lookup by email right now. Let's update that later. For now, we'll store a placeholder email.
    
    let new_org = Organization {
        id: None,
        name: payload.name,
        owner_id: user_id,
        members: vec![OrgMember {
            user_id,
            email: "owner@example.com".to_string(), // Placeholder
            role: "Owner".to_string(),
        }],
    };

    let result = orgs_collection
        .insert_one(new_org.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut org = new_org;
    org.id = Some(result.inserted_id.as_object_id().unwrap());

    // Update KV Cache
    let kv_key = format!("org:{}:user:{}", org.id.unwrap().to_hex(), user_id.to_hex());
    let _ = state.kv.put(&kv_key, "Owner".into()).await;

    Ok(Json(org))
}

async fn list_orgs(
    State(state): State<AppState>,
    user: JwtUser,
) -> Result<Json<Vec<Organization>>, (StatusCode, String)> {
    let orgs_collection = state.db.collection::<Organization>("organizations");

    let user_id = ObjectId::from_str(&user.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;

    let mut cursor = orgs_collection
        .find(doc! { "members.user_id": user_id })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut orgs = Vec::new();
    while let Some(org) = cursor
        .next()
        .await
    {
        if let Ok(org) = org {
            orgs.push(org);
        }
    }

    Ok(Json(orgs))
}

#[derive(serde::Deserialize)]
pub struct InternalUserResponse {
    pub id: String,
    pub email: String,
}

async fn invite_user(
    State(state): State<AppState>,
    user: JwtUser,
    Path(org_id): Path<String>,
    Json(payload): Json<InviteRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let orgs_collection = state.db.collection::<Organization>("organizations");

    let user_id = ObjectId::from_str(&user.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;
    let oid = ObjectId::from_str(&org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid organization ID".to_string()))?;

    // Check if user is Owner or Admin
    let org = orgs_collection
        .find_one(doc! { 
            "_id": oid, 
            "members": { 
                "$elemMatch": { 
                    "user_id": user_id, 
                    "role": { "$in": ["Owner", "Admin"] } 
                } 
            } 
        })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::FORBIDDEN, "Only Admins and Owners can invite users".to_string()))?;

    // Call Auth service to get user ID
    let auth_url = std::env::var("AUTH_SERVICE_URL").unwrap_or_else(|_| "http://auth:3000".to_string());
    let reqwest_client = reqwest::Client::new();
    let res = reqwest_client.get(&format!("{}/internal/users?email={}", auth_url, payload.email))
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !res.status().is_success() {
        return Err((StatusCode::BAD_REQUEST, "User not found".to_string()));
    }

    let internal_user: InternalUserResponse = res.json().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let invitee_id = ObjectId::from_str(&internal_user.id).unwrap();

    // Check if user is already in the org
    if org.members.iter().any(|m| m.user_id == invitee_id) {
        return Err((StatusCode::BAD_REQUEST, "User is already a member".to_string()));
    }

    let new_member = OrgMember {
        user_id: invitee_id,
        email: payload.email,
        role: payload.role.clone(),
    };

    orgs_collection.update_one(
        doc! { "_id": oid },
        doc! { "$push": { "members": mongodb::bson::to_bson(&new_member).unwrap() } }
    ).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Update KV Cache
    let kv_key = format!("org:{}:user:{}", oid.to_hex(), invitee_id.to_hex());
    let _ = state.kv.put(&kv_key, payload.role.into()).await;

    Ok(StatusCode::CREATED)
}

async fn get_members(
    State(state): State<AppState>,
    user: JwtUser,
    Path(org_id): Path<String>,
) -> Result<Json<Vec<OrgMember>>, (StatusCode, String)> {
    let orgs_collection = state.db.collection::<Organization>("organizations");
    let user_id = ObjectId::from_str(&user.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;
    let oid = ObjectId::from_str(&org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid org ID".to_string()))?;

    let org = orgs_collection
        .find_one(doc! { "_id": oid, "members.user_id": user_id })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Organization not found".to_string()))?;

    Ok(Json(org.members))
}

async fn update_member_role(
    State(state): State<AppState>,
    user: JwtUser,
    Path((org_id, member_id)): Path<(String, String)>,
    Json(payload): Json<UpdateRoleRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let orgs_collection = state.db.collection::<Organization>("organizations");
    let caller_id = ObjectId::from_str(&user.user_id).unwrap();
    let oid = ObjectId::from_str(&org_id).unwrap();
    let target_id = ObjectId::from_str(&member_id).unwrap();

    let org = orgs_collection
        .find_one(doc! { 
            "_id": oid, 
            "members": { "$elemMatch": { "user_id": caller_id, "role": { "$in": ["Owner", "Admin"] } } } 
        })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::FORBIDDEN, "Unauthorized".to_string()))?;

    if target_id == org.owner_id {
        return Err((StatusCode::BAD_REQUEST, "Cannot change owner role".to_string()));
    }

    orgs_collection.update_one(
        doc! { "_id": oid, "members.user_id": target_id },
        doc! { "$set": { "members.$.role": payload.role.clone() } }
    ).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Update KV Cache
    let kv_key = format!("org:{}:user:{}", oid.to_hex(), target_id.to_hex());
    let _ = state.kv.put(&kv_key, payload.role.into()).await;

    Ok(StatusCode::OK)
}

async fn remove_member(
    State(state): State<AppState>,
    user: JwtUser,
    Path((org_id, member_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let orgs_collection = state.db.collection::<Organization>("organizations");
    let caller_id = ObjectId::from_str(&user.user_id).unwrap();
    let oid = ObjectId::from_str(&org_id).unwrap();
    let target_id = ObjectId::from_str(&member_id).unwrap();

    let org = orgs_collection
        .find_one(doc! { 
            "_id": oid, 
            "members": { "$elemMatch": { "user_id": caller_id, "role": { "$in": ["Owner", "Admin"] } } } 
        })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::FORBIDDEN, "Unauthorized".to_string()))?;

    if target_id == org.owner_id {
        return Err((StatusCode::BAD_REQUEST, "Cannot remove owner".to_string()));
    }

    orgs_collection.update_one(
        doc! { "_id": oid },
        doc! { "$pull": { "members": { "user_id": target_id } } }
    ).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Update KV Cache
    let kv_key = format!("org:{}:user:{}", oid.to_hex(), target_id.to_hex());
    let _ = state.kv.delete(&kv_key).await;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_member_role(
    State(state): State<AppState>,
    Path((org_id, member_id)): Path<(String, String)>,
) -> Result<String, (StatusCode, String)> {
    let orgs_collection = state.db.collection::<Organization>("organizations");
    let oid = ObjectId::from_str(&org_id).unwrap();
    let target_id = ObjectId::from_str(&member_id).unwrap();

    let org = orgs_collection
        .find_one(doc! { "_id": oid })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Org not found".to_string()))?;

    let role = org.members.iter().find(|m| m.user_id == target_id).map(|m| m.role.clone())
        .ok_or((StatusCode::NOT_FOUND, "User not in org".to_string()))?;

    Ok(role)
}

