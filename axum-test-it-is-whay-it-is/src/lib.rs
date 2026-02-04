pub mod auth;
pub mod middleware;
pub mod modules;
pub mod shared;

use axum::{
    http::{Method, StatusCode},
    response::Json,
    Router,
};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use modules::{
    posts::{repository::create_post_repository, routes::create_post_routes},
    users::{repository::UserRepository, routes::create_user_routes, service::UserService},
};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
    pub user_service: UserService<UserRepository>,
    pub post_repository: modules::posts::repository::PostRepository,
}

pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub server_addr: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/axum_test".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "default_secret_key_at_least_32_characters_long".to_string()),
            server_addr: std::env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string()),
        })
    }
}

pub async fn create_app(config: AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&config.database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    tracing::info!("Database migrations completed");

    // Create repositories and services
    let user_repository = UserRepository::new(pool.clone());
    let user_service = UserService::new(user_repository, config.jwt_secret.clone());
    let post_repository = create_post_repository(pool.clone());

    // Create app state
    let state = AppState {
        db: pool,
        jwt_secret: config.jwt_secret,
        user_service,
        post_repository,
    };

    // Create router
    let app = create_router(state);

    // Bind server
    let listener = tokio::net::TcpListener::bind(&config.server_addr).await?;
    tracing::info!("Server listening on {}", config.server_addr);

    // Start server
    axum::serve(listener, app).await?;

    Ok(())
}

fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", axum::routing::get(health_check))
        // API routes
        .nest("/api/users", create_user_routes())
        .nest("/api/posts", create_post_routes())
        // 404 handler
        .fallback(not_found_handler)
        // Middleware
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers(tower_http::cors::Any),
        )
        .layer(TraceLayer::new_for_http())
        // State
        .with_state(state)
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn not_found_handler() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": "Not found" })),
    )
}

// Test helpers
#[cfg(any(test, feature = "test-helpers"))]
pub mod test_helpers {
    use super::*;

    pub async fn create_test_app(pool: PgPool) -> Router {
        let config = AppConfig {
            database_url: String::new(), // Not used in tests
            jwt_secret: "test_secret_key_at_least_32_characters_long".to_string(),
            server_addr: String::new(), // Not used in tests
        };

        let user_repository = UserRepository::new(pool.clone());
        let user_service = UserService::new(user_repository, config.jwt_secret.clone());
        let post_repository = create_post_repository(pool.clone());

        let state = AppState {
            db: pool,
            jwt_secret: config.jwt_secret,
            user_service,
            post_repository,
        };

        create_router(state)
    }
}
