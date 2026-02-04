/// Shared test utilities and helpers
/// This module provides a shared Postgres container with per-test database isolation

use sqlx::PgPool;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::ContainerAsync;
use tokio::sync::OnceCell;
use uuid::Uuid;

// Global container - started once, shared across tests
static CONTAINER: OnceCell<(ContainerAsync<Postgres>, String)> = OnceCell::const_new();

async fn ensure_container() -> &'static (ContainerAsync<Postgres>, String) {
    CONTAINER
        .get_or_init(|| async {
            let container = Postgres::default()
                .start()
                .await
                .expect("Failed to start postgres container");

            let host = container.get_host().await.expect("Failed to get host");
            let port = container
                .get_host_port_ipv4(5432)
                .await
                .expect("Failed to get port");

            let base_url = format!("postgres://postgres:postgres@{}:{}", host, port);
            (container, base_url)
        })
        .await
}

pub struct TestContext {
    pub pool: PgPool,
    db_name: String,
    base_url: String,
}

impl TestContext {
    pub async fn cleanup(self) {
        self.pool.close().await;

        // Connect to postgres database to drop test database
        let admin_url = format!("{}/postgres", self.base_url);
        if let Ok(admin_pool) = PgPool::connect(&admin_url).await {
            let _ = sqlx::query(&format!(
                "DROP DATABASE IF EXISTS {} WITH (FORCE)",
                self.db_name
            ))
            .execute(&admin_pool)
            .await;
            admin_pool.close().await;
        }
    }
}

/// Sets up a test database with a unique name for isolation
/// Returns a TestContext that should be cleaned up after the test
pub async fn setup_test_db() -> TestContext {
    let (_, base_url) = ensure_container().await;

    // Create unique database name
    let db_name = format!("test_{}", Uuid::new_v4().to_string().replace("-", "_"));

    // Connect to postgres database to create test database
    let admin_url = format!("{}/postgres", base_url);
    let admin_pool = PgPool::connect(&admin_url)
        .await
        .expect("Failed to connect to admin database");

    sqlx::query(&format!("CREATE DATABASE {}", db_name))
        .execute(&admin_pool)
        .await
        .expect("Failed to create test database");

    admin_pool.close().await;

    // Connect to new test database
    let test_url = format!("{}/{}", base_url, db_name);
    let pool = PgPool::connect(&test_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    TestContext {
        pool,
        db_name,
        base_url: base_url.clone(),
    }
}
