use surreal_db::Database;
use tracing::info;

use crate::schema::SCHEMA_DEFINITIONS;
use crate::seed::seed_database;

pub struct MigrationRunner {
    db: Database,
}

impl MigrationRunner {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        info!("Running database migrations");
        self.apply_schema().await?;
        info!("Migrations completed successfully");
        Ok(())
    }

    pub async fn run_with_seed(&self) -> anyhow::Result<()> {
        info!("Running database migrations with seed data");
        self.apply_schema().await?;
        seed_database(&self.db).await?;
        info!("Migrations and seeding completed successfully");
        Ok(())
    }

    async fn apply_schema(&self) -> anyhow::Result<()> {
        info!("Applying schema definitions");

        for (index, definition) in SCHEMA_DEFINITIONS.iter().enumerate() {
            info!(
                "Applying schema definition {}/{}",
                index + 1,
                SCHEMA_DEFINITIONS.len()
            );
            self.db.client.query(*definition).await?;
        }

        info!("Schema definitions applied successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_migrations() {
        let db = Database::new_in_memory().await.unwrap();
        let runner = MigrationRunner::new(db);

        let result = runner.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_migrations_with_seed() {
        let db = Database::new_in_memory().await.unwrap();
        let runner = MigrationRunner::new(db);

        let result = runner.run_with_seed().await;
        assert!(result.is_ok());
    }
}
