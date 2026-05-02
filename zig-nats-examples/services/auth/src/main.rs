mod models;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use config::AppConfig;
use async_nats::service::ServiceExt;
use futures::StreamExt;
use bytes::Bytes;
use mongodb::{bson::doc, options::IndexOptions, IndexModel};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::{Utc, Duration};
use models::*;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    db: mongodb::Database,
    nats: async_nats::Client,
    config: AppConfig,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "auth=debug,info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Auth Service...");

    // Load configuration
    let settings = AppConfig::load().map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;
    tracing::info!("Configuration loaded");

    // Connect to NATS
    let nats_client = async_nats::connect(&settings.nats.url).await?;
    tracing::info!("Connected to NATS at {}", settings.nats.url);

    // Initialize NATS Microservice
    let service = nats_client
        .service_builder()
        .description("Auth Service")
        .start("auth", "0.1.0")
        .await
        .map_err(|e| anyhow::anyhow!("failed to start NATS service: {}", e))?;
    
    tracing::info!("NATS Microservice 'auth' started");

    // Use a group for namespacing subjects
    let group = service.group("auth");
    let mut ping_endpoint = group.endpoint("ping").await
        .map_err(|e| anyhow::anyhow!("failed to add NATS endpoint: {}", e))?;
    
    // Process NATS service requests in a background task
    tokio::spawn(async move {
        tracing::info!("NATS 'auth.ping' endpoint ready");
        while let Some(request) = ping_endpoint.next().await {
            tracing::debug!("Received NATS ping request");
            if let Err(e) = request.respond(Ok(Bytes::from("pong"))).await {
                tracing::error!("Failed to respond to NATS request: {}", e);
            }
        }
    });

    // Connect to MongoDB
    let mongo_config = settings.mongodb.as_ref().ok_or_else(|| anyhow::anyhow!("MongoDB configuration is missing"))?;
    let mongo_client = mongodb::Client::with_uri_str(&mongo_config.url).await?;
    let db = mongo_client.database(&mongo_config.db_name);
    
    // Ensure unique index on email
    let users_collection = db.collection::<User>("users");
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "email": 1 })
        .options(options)
        .build();
    users_collection.create_index(model).await?;
    tracing::info!("Unique index on email ensured");

    let state = AppState {
        db,
        nats: nats_client,
        config: settings.clone(),
    };

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/register", post(register))
        .route("/login", post(login))
        .layer(CorsLayer::permissive())
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

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<User>, (StatusCode, String)> {
    let users_collection = state.db.collection::<User>("users");

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password".to_string()))?
        .to_string();

    let new_user = User {
        id: None,
        email: payload.email,
        password_hash,
    };

    match users_collection.insert_one(new_user.clone()).await {
        Ok(result) => {
            let mut user = new_user;
            user.id = Some(result.inserted_id.as_object_id().unwrap());
            Ok(Json(user))
        }
        Err(e) => {
            if e.to_string().contains("E11000") {
                Err((StatusCode::CONFLICT, "Email already exists".to_string()))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
            }
        }
    }
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let users_collection = state.db.collection::<User>("users");

    let user = users_collection
        .find_one(doc! { "email": &payload.email })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    // Verify password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid password hash format".to_string()))?;
    
    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    // Generate JWT
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user.id.unwrap().to_hex(),
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.auth.as_ref().expect("Auth config missing").jwt_secret.as_ref()),
    ).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate token".to_string()))?;

    Ok(Json(AuthResponse { token }))
}
