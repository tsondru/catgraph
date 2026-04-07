use surrealdb::engine::local::Db;
use surrealdb::types::RecordId;
use surrealdb::Surreal;
use crate::error::PersistError;
use crate::query::QueryHelper;
use crate::types_v2::{GraphEdgeRecord, GraphNodeRecord};
use crate::utils::IdOnly;

/// Store for pairwise RELATE edges in the V2 schema.
pub struct EdgeStore<'a> {
    db: &'a Surreal<Db>,
}

impl<'a> EdgeStore<'a> {
    #[must_use] 
    pub fn new(db: &'a Surreal<Db>) -> Self {
        Self { db }
    }

    /// Create a RELATE edge between two `graph_node` records.
    ///
    /// Uses raw `.query("RELATE ...")` — the SDK's `.insert().relation()` serializes
    /// `RecordId` incorrectly via `serde_json::json!`.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::InvalidData`] if the RELATE statement fails to
    /// return a record. Returns [`PersistError::Surreal`] on database errors.
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

    /// Get an edge by `RecordId`.
    ///
    /// Uses an explicit field list instead of `db.select()` to avoid
    /// deserializing the system-managed `in`/`out` relation fields which
    /// the `SurrealValue` derive cannot handle with `#[serde(rename)]`.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::NotFound`] if no edge exists for the given ID.
    /// Returns [`PersistError::Surreal`] on database errors.
    pub async fn get(&self, id: &RecordId) -> Result<GraphEdgeRecord, PersistError> {
        let mut result = self
            .db
            .query("SELECT id, kind, weight, properties FROM $edge_id")
            .bind(("edge_id", id.clone()))
            .await?;
        let record: Option<GraphEdgeRecord> = result.take(0)?;
        record.ok_or_else(|| PersistError::NotFound(format!("{id:?}")))
    }

    /// Delete an edge by `RecordId`.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::Surreal`] if the database operation fails.
    pub async fn delete(&self, id: &RecordId) -> Result<(), PersistError> {
        self.db
            .query("DELETE $edge_id")
            .bind(("edge_id", id.clone()))
            .await?;
        Ok(())
    }

    /// Traverse outbound edges of a given kind from a node, returning connected nodes.
    ///
    /// Delegates to [`QueryHelper::outbound_neighbors`].
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::NotFound`] if a referenced target node no
    /// longer exists. Returns [`PersistError::Surreal`] on database errors.
    pub async fn traverse_outbound(
        &self,
        from: &RecordId,
        edge_kind: &str,
    ) -> Result<Vec<GraphNodeRecord>, PersistError> {
        QueryHelper::new(self.db).outbound_neighbors(from, edge_kind).await
    }

    /// Traverse inbound edges of a given kind to a node, returning source nodes.
    ///
    /// Delegates to [`QueryHelper::inbound_neighbors`].
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::NotFound`] if a referenced source node no
    /// longer exists. Returns [`PersistError::Surreal`] on database errors.
    pub async fn traverse_inbound(
        &self,
        to: &RecordId,
        edge_kind: &str,
    ) -> Result<Vec<GraphNodeRecord>, PersistError> {
        QueryHelper::new(self.db).inbound_neighbors(to, edge_kind).await
    }

    /// Find all edges between two specific nodes.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::Surreal`] if the database operation fails.
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

