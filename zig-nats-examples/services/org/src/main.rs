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

    // Connect to NATS
    let nats_client = async_nats::connect(&settings.nats.url).await?;
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

    // Connect to MongoDB
    let mongo_config = settings.mongodb.as_ref().ok_or_else(|| anyhow::anyhow!("MongoDB configuration is missing"))?;
    let mongo_client = mongodb::Client::with_uri_str(&mongo_config.url).await?;
    let db = mongo_client.database(&mongo_config.db_name);
    tracing::info!("Connected to MongoDB");

    let state = AppState {
        db,
        nats: nats_client,
        config: settings.clone(),
    };

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/orgs", post(create_org).get(list_orgs))
        .route("/orgs/:id/invite", post(invite_user))
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
    let orgs_collection = state.db.collection::<Organization>("organizations");

    let user_id = ObjectId::from_str(&user.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;

    let new_org = Organization {
        id: None,
        name: payload.name,
        owner_id: user_id,
        member_ids: vec![user_id],
    };

    let result = orgs_collection
        .insert_one(new_org.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut org = new_org;
    org.id = Some(result.inserted_id.as_object_id().unwrap());

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
        .find(doc! { "member_ids": user_id })
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

async fn invite_user(
    State(state): State<AppState>,
    user: JwtUser,
    Path(org_id): Path<String>,
    Json(payload): Json<InviteRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let orgs_collection = state.db.collection::<Organization>("organizations");
    let invites_collection = state.db.collection::<Invitation>("invitations");

    let user_id = ObjectId::from_str(&user.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;
    let oid = ObjectId::from_str(&org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid organization ID".to_string()))?;

    // Check if user is the owner
    let org = orgs_collection
        .find_one(doc! { "_id": oid, "owner_id": user_id })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::FORBIDDEN, "Only the owner can invite users".to_string()))?;

    let invitation = Invitation {
        id: None,
        org_id: org.id.unwrap(),
        inviter_id: user_id,
        invitee_email: payload.email,
        status: InvitationStatus::Pending,
    };

    invites_collection
        .insert_one(invitation)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // In a real app, we would publish a NATS event here to notify the user
    // state.nats.publish("org.invitation.created", ...).await?;

    Ok(StatusCode::CREATED)
}
