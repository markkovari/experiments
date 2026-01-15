use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Invalid email format: {0}")]
    InvalidEmail(String),

    #[error("Invalid phone number format: {0}")]
    InvalidPhone(String),

    #[error("Invalid date: {0}")]
    InvalidDate(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
