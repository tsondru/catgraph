use serde::{Deserialize, Serialize};
use surrealdb::types::RecordId;
use surrealdb_types::SurrealValue;

/// A first-class graph vertex in the V2 schema.
#[derive(Debug, Serialize, Deserialize, Clone, SurrealValue)]
pub struct GraphNodeRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub properties: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f64>>,
}

/// A pairwise RELATE edge between two graph_node records.
///
/// Omits `in`/`out` fields because the `SurrealValue` derive does not support
/// `#[serde(rename)]`, and these system-managed relation fields are not
/// consistently present in all query/select contexts. Use `EdgeStore` traversal
/// methods to resolve connected nodes.
#[derive(Debug, Serialize, Deserialize, Clone, SurrealValue)]
pub struct GraphEdgeRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    #[serde(default)]
    pub properties: serde_json::Value,
}

/// Hub record for an n-ary hyperedge (reification node).
#[derive(Debug, Serialize, Deserialize, Clone, SurrealValue)]
pub struct HyperedgeHubRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub kind: String,
    #[serde(default)]
    pub properties: serde_json::Value,
    pub source_count: i64,
    pub target_count: i64,
    /// Schema-level REFERENCE to parent hubs for composition provenance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_hubs: Option<Vec<RecordId>>,
    /// Computed field: true when parent_hubs is non-empty (v3.0.5 selective evaluation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_provenance: Option<bool>,
}

/// Composition relation edge: tracks which operation produced a child hub from a parent.
#[derive(Debug, Serialize, Deserialize, Clone, SurrealValue)]
pub struct ComposedFromRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub operation: String,
}

/// Source participation edge: graph_node -> hyperedge_hub.
#[derive(Debug, Serialize, Deserialize, Clone, SurrealValue)]
pub struct SourceOfRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    #[serde(rename = "in")]
    pub in_node: RecordId,
    #[serde(rename = "out")]
    pub out_hub: RecordId,
    pub position: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<String>, // decimal stored as string for exact representation
}

/// Target participation edge: hyperedge_hub -> graph_node.
#[derive(Debug, Serialize, Deserialize, Clone, SurrealValue)]
pub struct TargetOfRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    #[serde(rename = "in")]
    pub in_hub: RecordId,
    #[serde(rename = "out")]
    pub out_node: RecordId,
    pub position: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<String>, // decimal stored as string for exact representation
}

/// A Petri net record.
#[derive(Debug, Serialize, Deserialize, Clone, SurrealValue)]
pub struct PetriNetRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub name: String,
    pub label_type: String,
    #[serde(default)]
    pub properties: serde_json::Value,
}

/// A place in a Petri net.
#[derive(Debug, Serialize, Deserialize, Clone, SurrealValue)]
pub struct PetriPlaceRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub net: RecordId,
    pub position: i64,
    pub label: String,
    pub label_type: String,
    #[serde(default)]
    pub properties: serde_json::Value,
}

/// A transition in a Petri net.
#[derive(Debug, Serialize, Deserialize, Clone, SurrealValue)]
pub struct PetriTransitionRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub net: RecordId,
    pub position: i64,
    #[serde(default)]
    pub properties: serde_json::Value,
}

/// A marking snapshot (token distribution).
#[derive(Debug, Serialize, Deserialize, Clone, SurrealValue)]
pub struct MarkingRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub net: RecordId,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub tokens: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<i64>,
}

/// A node in a multiway evolution graph, representing a state at a given branch and step.
#[derive(Debug, Serialize, Deserialize, Clone, SurrealValue)]
pub struct MultiwayNodeRecord {
    /// Record ID assigned by the database.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    /// Branch identifier (maps to `BranchId` in catgraph core).
    pub branch_id: i64,
    /// Step/depth in the evolution.
    pub step: i64,
    /// Human-readable state label.
    pub state_label: String,
    /// Optional properties bag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<serde_json::Value>,
}

/// An edge in a multiway evolution graph (evolution or branchial).
///
/// Omits `in`/`out` fields — see [`GraphEdgeRecord`] for rationale.
#[derive(Debug, Serialize, Deserialize, Clone, SurrealValue)]
pub struct MultiwayEdgeRecord {
    /// Record ID assigned by the database.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    /// Edge classification: `"evolution"` or `"branchial"`.
    pub edge_type: String,
    /// Optional properties bag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<serde_json::Value>,
}
