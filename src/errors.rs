use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatgraphError {
    Composition(String),
    Interpret(String),
    Operadic(String),
    Relation(String),
}

impl fmt::Display for CatgraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CatgraphError::Composition(s) => write!(f, "Composition error: {s}"),
            CatgraphError::Interpret(s) => write!(f, "Interpret error: {s}"),
            CatgraphError::Operadic(s) => write!(f, "Operadic error: {s}"),
            CatgraphError::Relation(s) => write!(f, "Relation error: {s}"),
        }
    }
}

impl std::error::Error for CatgraphError {}
