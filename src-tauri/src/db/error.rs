use rusqlite;
use thiserror::Error;

/// Unified error type for all repository operations.
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Duplicate: {0}")]
    Duplicate(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Pool error: {0}")]
    Pool(#[from] r2d2::Error),
}
