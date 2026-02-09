use axum_idiomatic_validate_and_user_context::{build_router, config::Config, db, AppState};
use tower_http::trace::TraceLayer;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let config = Config::from_env();
    let pool = db::create_pool(&config.database_url).await;
    db::run_migrations(&pool).await;

    let state = AppState {
        pool,
        jwt_secret: config.jwt_secret,
    };

    let app: axum::Router = build_router(state).layer(TraceLayer::new_for_http());

    let listener: tokio::net::TcpListener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
