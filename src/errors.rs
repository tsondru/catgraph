use thiserror::Error;

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
}
