use serde::{Deserialize, Serialize};
use surrealdb::types::RecordId;
use surrealdb_types::SurrealValue;

/// `SurrealDB` record type for a persisted Cospan.
#[derive(Debug, Serialize, Deserialize, Clone, SurrealValue)]
pub struct CospanRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub left_map: Vec<i64>,
    pub right_map: Vec<i64>,
    pub middle_labels: Vec<String>,
    pub label_type: String,
    pub is_left_id: bool,
    pub is_right_id: bool,
}

/// `SurrealDB` record type for a persisted `NamedCospan`.
#[derive(Debug, Serialize, Deserialize, Clone, SurrealValue)]
pub struct NamedCospanRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub cospan_ref: RecordId,
    pub left_names: Vec<String>,
    pub right_names: Vec<String>,
}

/// `SurrealDB` record type for a persisted Span.
#[derive(Debug, Serialize, Deserialize, Clone, SurrealValue)]
pub struct SpanRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub left_labels: Vec<String>,
    pub right_labels: Vec<String>,
    pub middle_pairs: Vec<Vec<i64>>,
    pub label_type: String,
    pub is_left_id: bool,
    pub is_right_id: bool,
}
