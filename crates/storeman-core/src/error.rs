use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoremanError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Authentication failed")]
    AuthFailed,
    #[error("Insufficient permissions: {0}")]
    Unauthorized(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Insufficient stock: available {available}, requested {requested}")]
    InsufficientStock { available: i64, requested: i64 },
    #[error("Audit chain broken at entry {0}")]
    AuditChainBroken(i64),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, StoremanError>;
