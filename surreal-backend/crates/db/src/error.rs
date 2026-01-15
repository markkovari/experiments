use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("SurrealDB error: {0}")]
    SurrealDb(#[from] surrealdb::Error),

    #[error("Record not found: {0}")]
    NotFound(String),

    #[error("Record already exists: {0}")]
    AlreadyExists(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Core validation error: {0}")]
    CoreError(#[from] surreal_core::CoreError),

    #[error("Database error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, DbError>;
