#![allow(dead_code)]

use axum_idiomatic_validate_and_user_context::{build_router, db, AppState};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn test_pool() -> PgPool {
    dotenvy::dotenv().ok();
    let base_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for integration tests");
    let db_name = format!("test_{}", Uuid::new_v4().to_string().replace('-', ""));

    // Connect to default database to create test database
    let admin_pool = PgPool::connect(&base_url).await.unwrap();
    sqlx::raw_sql(&format!("CREATE DATABASE \"{db_name}\""))
        .execute(&admin_pool)
        .await
        .unwrap();
    admin_pool.close().await;

    // Connect to the new test database
    let test_url = base_url
        .rsplit_once('/')
        .map(|(base, _)| format!("{base}/{db_name}"))
        .unwrap();

    let pool = PgPool::connect(&test_url).await.unwrap();
    db::run_migrations(&pool).await;
    pool
}

pub async fn insert_test_user(pool: &PgPool, username: &str) -> (uuid::Uuid, String) {
    use axum_idiomatic_validate_and_user_context::models::user::create_user;

    let user = create_user(pool, username, &format!("{username}@test.com"), "password123")
        .await
        .unwrap();

    let token =
        axum_idiomatic_validate_and_user_context::auth::encode_jwt("dev-secret", user.id, user.role);
    (user.id, token)
}

pub async fn insert_test_admin(pool: &PgPool, username: &str) -> (uuid::Uuid, String) {
    use axum_idiomatic_validate_and_user_context::models::user::{create_user, Role};

    let user = create_user(pool, username, &format!("{username}@test.com"), "password123")
        .await
        .unwrap();

    sqlx::raw_sql(&format!(
        "UPDATE users SET role = 'admin' WHERE id = '{}'",
        user.id
    ))
    .execute(pool)
    .await
    .unwrap();

    let token =
        axum_idiomatic_validate_and_user_context::auth::encode_jwt("dev-secret", user.id, Role::Admin);
    (user.id, token)
}

pub async fn spawn_app() -> (String, PgPool) {
    let pool = test_pool().await;
    let state = AppState {
        pool: pool.clone(),
        jwt_secret: "dev-secret".into(),
    };

    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://{addr}"), pool)
}
