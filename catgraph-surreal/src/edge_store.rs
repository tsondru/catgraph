use surrealdb::engine::local::Db;
use surrealdb::types::RecordId;
use surrealdb::Surreal;
use crate::error::PersistError;
use crate::node_store::NodeStore;
use crate::types_v2::{GraphEdgeRecord, GraphNodeRecord};
use crate::utils::{IdOnly, InRef, OutRef};

/// Store for pairwise RELATE edges in the V2 schema.
pub struct EdgeStore<'a> {
    db: &'a Surreal<Db>,
    node_store: NodeStore<'a>,
}

impl<'a> EdgeStore<'a> {
    pub fn new(db: &'a Surreal<Db>) -> Self {
        Self {
            db,
            node_store: NodeStore::new(db),
        }
    }

    /// Create a RELATE edge between two graph_node records.
    ///
    /// Uses raw `.query("RELATE ...")` — the SDK's `.insert().relation()` serializes
    /// RecordId incorrectly via `serde_json::json!`.
    pub async fn relate(
        &self,
        from: &RecordId,
        to: &RecordId,
        kind: &str,
        weight: Option<f64>,
        properties: serde_json::Value,
    ) -> Result<RecordId, PersistError> {
        let mut result = self
            .db
            .query(
                "RELATE $from->graph_edge->$to SET kind = $kind, weight = $weight, properties = $properties RETURN id",
            )
            .bind(("from", from.clone()))
            .bind(("to", to.clone()))
            .bind(("kind", kind.to_string()))
            .bind(("weight", weight))
            .bind(("properties", properties))
            .await?;
        let created: Option<IdOnly> = result.take(0)?;
        let created = created
            .ok_or_else(|| PersistError::InvalidData("failed to create graph_edge".into()))?;
        Ok(created.id)
    }

    /// Get an edge by RecordId.
    ///
    /// Uses an explicit field list instead of `db.select()` to avoid
    /// deserializing the system-managed `in`/`out` relation fields which
    /// the `SurrealValue` derive cannot handle with `#[serde(rename)]`.
    pub async fn get(&self, id: &RecordId) -> Result<GraphEdgeRecord, PersistError> {
        let mut result = self
            .db
            .query("SELECT id, kind, weight, properties FROM $edge_id")
            .bind(("edge_id", id.clone()))
            .await?;
        let record: Option<GraphEdgeRecord> = result.take(0)?;
        record.ok_or_else(|| PersistError::NotFound(format!("{id:?}")))
    }

    /// Delete an edge by RecordId.
    pub async fn delete(&self, id: &RecordId) -> Result<(), PersistError> {
        self.db
            .query("DELETE $edge_id")
            .bind(("edge_id", id.clone()))
            .await?;
        Ok(())
    }

    /// Traverse outbound edges of a given kind from a node, returning connected nodes.
    ///
    /// Queries the `graph_edge` table directly (avoiding `serde_json::Value` intermediary
    /// which cannot deserialize `RecordId`), then fetches each target node by id.
    pub async fn traverse_outbound(
        &self,
        from: &RecordId,
        edge_kind: &str,
    ) -> Result<Vec<GraphNodeRecord>, PersistError> {
        let mut result = self
            .db
            .query("SELECT out FROM graph_edge WHERE in = $from AND kind = $kind")
            .bind(("from", from.clone()))
            .bind(("kind", edge_kind.to_string()))
            .await?;
        let edges: Vec<OutRef> = result.take(0)?;
        let mut nodes = Vec::with_capacity(edges.len());
        for edge in &edges {
            nodes.push(self.node_store.get(&edge.out).await?);
        }
        Ok(nodes)
    }

    /// Traverse inbound edges of a given kind to a node, returning source nodes.
    ///
    /// Queries the `graph_edge` table directly, then fetches each source node by id.
    pub async fn traverse_inbound(
        &self,
        to: &RecordId,
        edge_kind: &str,
    ) -> Result<Vec<GraphNodeRecord>, PersistError> {
        let mut result = self
            .db
            .query("SELECT `in` AS src FROM graph_edge WHERE out = $to AND kind = $kind")
            .bind(("to", to.clone()))
            .bind(("kind", edge_kind.to_string()))
            .await?;
        let edges: Vec<InRef> = result.take(0)?;
        let mut nodes = Vec::with_capacity(edges.len());
        for edge in &edges {
            nodes.push(self.node_store.get(&edge.src).await?);
        }
        Ok(nodes)
    }

    /// Find all edges between two specific nodes.
    pub async fn edges_between(
        &self,
        from: &RecordId,
        to: &RecordId,
    ) -> Result<Vec<GraphEdgeRecord>, PersistError> {
        let mut result = self
            .db
            .query("SELECT id, kind, weight, properties FROM graph_edge WHERE `in` = $from AND out = $to")
            .bind(("from", from.clone()))
            .bind(("to", to.clone()))
            .await?;
        let records: Vec<GraphEdgeRecord> = result.take(0)?;
        Ok(records)
    }
}

