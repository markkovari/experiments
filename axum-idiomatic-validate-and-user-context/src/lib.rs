use sqlx::PgPool;

pub mod auth;
pub mod config;
pub mod db;
pub mod errors;
pub mod models;
pub mod routes;
pub mod validate;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
}

pub fn build_router(state: AppState) -> axum::Router {
    routes::app_router(state)
}
