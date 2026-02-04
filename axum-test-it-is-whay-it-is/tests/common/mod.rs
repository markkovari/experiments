/// Shared test utilities and helpers
/// This module is not run as a test itself but provides common functionality

use sqlx::PgPool;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::runners::AsyncRunner;

/// Sets up a test database with testcontainers
/// Returns the container (must be kept alive) and a connection pool
pub async fn setup_test_db() -> (testcontainers_modules::testcontainers::ContainerAsync<Postgres>, PgPool) {
    let container = Postgres::default()
        .start()
        .await
        .expect("Failed to start postgres container");

    let host = container.get_host().await.expect("Failed to get host");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("Failed to get port");

    let database_url = format!(
        "postgres://postgres:postgres@{}:{}/postgres",
        host.to_string(),
        port
    );

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    (container, pool)
}
