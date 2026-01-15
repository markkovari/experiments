use surrealdb::engine::any::Any;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tracing::info;

use crate::error::Result;

#[derive(Clone)]
pub struct Database {
    pub client: Surreal<Any>,
}

impl Database {
    pub async fn new(path: &str) -> Result<Self> {
        info!("Initializing embedded SurrealDB at: {}", path);

        let client = surrealdb::engine::any::connect(format!("rocksdb://{}", path)).await?;
        client.use_ns("veterinary").use_db("clinic").await?;

        info!("Database initialized successfully");

        Ok(Self { client })
    }

    pub async fn new_in_memory() -> Result<Self> {
        info!("Initializing in-memory SurrealDB");

        let client = surrealdb::engine::any::connect("mem://").await?;
        client.use_ns("veterinary").use_db("clinic").await?;

        info!("In-memory database initialized successfully");

        Ok(Self { client })
    }

    pub async fn connect_remote(url: &str, username: &str, password: &str) -> Result<Self> {
        info!("Connecting to remote SurrealDB at: {}", url);

        let client = surrealdb::engine::any::connect(url).await?;
        client.signin(Root { username, password }).await?;
        client.use_ns("veterinary").use_db("clinic").await?;

        info!("Connected to remote database successfully");

        Ok(Self { client })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_database() {
        let db = Database::new_in_memory().await.unwrap();
        assert!(db.client.health().await.is_ok());
    }
}
