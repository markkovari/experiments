use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> PgPool {
    PgPool::connect(database_url)
        .await
        .expect("Failed to connect to database")
}

pub async fn run_migrations(pool: &PgPool) {
    let init = include_str!("../migrations/001_init.sql");
    sqlx::raw_sql(init)
        .execute(pool)
        .await
        .expect("Failed to run migrations");

    let add_role = include_str!("../migrations/002_add_user_role.sql");
    sqlx::raw_sql(add_role)
        .execute(pool)
        .await
        .expect("Failed to run role migration");
}
