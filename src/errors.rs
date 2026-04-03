use thiserror::Error;

use crate::finset::{TryFromFinSetError, TryFromInjError, TryFromSurjError};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CatgraphError {
    #[error("composition error: interface size mismatch (expected {expected}, got {actual})")]
    CompositionSizeMismatch { expected: usize, actual: usize },

    #[error("composition error: label mismatch at index {index} (expected {expected:?}, got {actual:?})")]
    CompositionLabelMismatch {
        index: usize,
        expected: String,
        actual: String,
    },

    #[error("composition error: {message}")]
    Composition { message: String },

    #[error("interpret error: {context}")]
    Interpret { context: String },

    #[error("operadic error: {message}")]
    Operadic { message: String },

    #[error("relation error: {message}")]
    Relation { message: String },

    #[error("finite set error: {message}")]
    FinSet { message: String },
}

impl From<TryFromSurjError> for CatgraphError {
    fn from(e: TryFromSurjError) -> Self {
        Self::FinSet { message: e.to_string() }
    }
}

impl From<TryFromInjError> for CatgraphError {
    fn from(e: TryFromInjError) -> Self {
        Self::FinSet { message: e.to_string() }
    }
}

impl From<TryFromFinSetError> for CatgraphError {
    fn from(e: TryFromFinSetError) -> Self {
        Self::FinSet { message: e.to_string() }
    }
}
