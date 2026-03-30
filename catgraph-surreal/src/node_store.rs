use surrealdb::engine::local::Db;
use surrealdb::types::RecordId;
use surrealdb::Surreal;

use crate::error::PersistError;
use crate::types_v2::GraphNodeRecord;

/// Store for first-class graph vertices in the V2 schema.
pub struct NodeStore<'a> {
    db: &'a Surreal<Db>,
}

impl<'a> NodeStore<'a> {
    pub fn new(db: &'a Surreal<Db>) -> Self {
        Self { db }
    }

    /// Create a new graph node, returning its RecordId.
    pub async fn create(
        &self,
        name: &str,
        kind: &str,
        labels: Vec<String>,
        properties: serde_json::Value,
    ) -> Result<RecordId, PersistError> {
        let record = GraphNodeRecord {
            id: None,
            name: name.to_string(),
            kind: kind.to_string(),
            labels,
            properties,
        };
        let created: Option<GraphNodeRecord> =
            self.db.create("graph_node").content(record).await?;
        let created = created
            .ok_or_else(|| PersistError::InvalidData("failed to create graph_node".into()))?;
        created
            .id
            .ok_or_else(|| PersistError::InvalidData("created node has no id".into()))
    }

    /// Get a graph node by RecordId.
    pub async fn get(&self, id: &RecordId) -> Result<GraphNodeRecord, PersistError> {
        let record: Option<GraphNodeRecord> = self.db.select(id).await?;
        record.ok_or_else(|| PersistError::NotFound(format!("{id:?}")))
    }

    /// Update an existing graph node's fields.
    ///
    /// Uses `.merge()` rather than `.content()` so that server-managed fields
    /// like `created_at` are preserved.
    pub async fn update(
        &self,
        id: &RecordId,
        name: &str,
        kind: &str,
        labels: Vec<String>,
        properties: serde_json::Value,
    ) -> Result<GraphNodeRecord, PersistError> {
        let patch = serde_json::json!({
            "name": name,
            "kind": kind,
            "labels": labels,
            "properties": properties,
        });
        let updated: Option<GraphNodeRecord> = self.db.update(id).merge(patch).await?;
        updated.ok_or_else(|| PersistError::NotFound(format!("{id:?}")))
    }

    /// Delete a graph node by RecordId.
    pub async fn delete(&self, id: &RecordId) -> Result<(), PersistError> {
        let _: Option<GraphNodeRecord> = self.db.delete(id).await?;
        Ok(())
    }

    /// Find all graph nodes matching a given kind.
    pub async fn find_by_kind(&self, kind: &str) -> Result<Vec<GraphNodeRecord>, PersistError> {
        let mut result = self
            .db
            .query("SELECT * FROM graph_node WHERE kind = $kind")
            .bind(("kind", kind.to_string()))
            .await?;
        let records: Vec<GraphNodeRecord> = result.take(0)?;
        Ok(records)
    }

    /// Find all graph nodes matching a given name.
    pub async fn find_by_name(&self, name: &str) -> Result<Vec<GraphNodeRecord>, PersistError> {
        let mut result = self
            .db
            .query("SELECT * FROM graph_node WHERE name = $name")
            .bind(("name", name.to_string()))
            .await?;
        let records: Vec<GraphNodeRecord> = result.take(0)?;
        Ok(records)
    }

    /// List all graph node RecordIds.
    pub async fn list(&self) -> Result<Vec<RecordId>, PersistError> {
        let records: Vec<GraphNodeRecord> = self.db.select("graph_node").await?;
        Ok(records.into_iter().filter_map(|r| r.id).collect())
    }
}
