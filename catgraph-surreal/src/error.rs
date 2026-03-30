use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersistError {
    #[error("SurrealDB error: {0}")]
    Surreal(#[from] surrealdb::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: String, got: String },

    #[error("record not found: {0}")]
    NotFound(String),

    #[error("invalid data: {0}")]
    InvalidData(String),

    #[error("transaction conflict (retryable): {0}")]
    TransactionConflict(String),
}

impl PersistError {
    /// Returns `true` if the error represents a retryable transaction conflict.
    pub fn is_transaction_conflict(&self) -> bool {
        match self {
            PersistError::TransactionConflict(_) => true,
            PersistError::Surreal(e) => {
                use surrealdb_types::QueryError;
                matches!(e.query_details(), Some(QueryError::TransactionConflict))
            }
            _ => false,
        }
    }
}
