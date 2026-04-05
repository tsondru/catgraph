
use surrealdb::engine::local::Db;
use surrealdb::types::RecordId;
use surrealdb::Surreal;
use surrealdb_types::SurrealValue;

use crate::error::PersistError;
use crate::node_store::NodeStore;
use crate::persist::Persistable;
use crate::types_v2::{GraphNodeRecord, HyperedgeHubRecord};

pub mod decompose;
pub mod provenance;
pub mod reconstruct;

/// Store for n-ary hyperedges using hub-node reification in the V2 schema.
///
/// Decomposes catgraph's `Cospan` and `Span` types into:
/// - A hub record (`hyperedge_hub`) representing the hyperedge
/// - `source_of` RELATE edges from source nodes to the hub
/// - `target_of` RELATE edges from the hub to target nodes
pub struct HyperedgeStore<'a> {
    db: &'a Surreal<Db>,
    pub(super) node_store: NodeStore<'a>,
}

impl<'a> HyperedgeStore<'a> {
    pub fn new(db: &'a Surreal<Db>) -> Self {
        Self {
            db,
            node_store: NodeStore::new(db),
        }
    }

    /// Get the hub record itself.
    pub async fn get_hub(&self, hub_id: &RecordId) -> Result<HyperedgeHubRecord, PersistError> {
        let record: Option<HyperedgeHubRecord> = self.db.select(hub_id).await?;
        record.ok_or_else(|| PersistError::NotFound(format!("{hub_id:?}")))
    }

    /// Delete a hub and all its source_of/target_of/composed_from edges.
    pub async fn delete_hub(&self, hub_id: &RecordId) -> Result<(), PersistError> {
        // Delete participation edges first
        self.db
            .query("DELETE source_of WHERE out = $hub")
            .bind(("hub", hub_id.clone()))
            .await?;
        self.db
            .query("DELETE target_of WHERE in = $hub")
            .bind(("hub", hub_id.clone()))
            .await?;
        // Delete composition relation edges (both directions)
        self.db
            .query("DELETE composed_from WHERE in = $hub OR out = $hub")
            .bind(("hub", hub_id.clone()))
            .await?;
        // Delete the hub itself (triggers ON DELETE UNSET for parent_hubs references)
        let _: Option<HyperedgeHubRecord> = self.db.delete(hub_id).await?;
        Ok(())
    }

    // --- internal helpers (pub(crate) so sub-modules can call them) ---

    pub(crate) async fn create_hub(
        &self,
        kind: &str,
        properties: serde_json::Value,
        source_count: i64,
        target_count: i64,
    ) -> Result<RecordId, PersistError> {
        let record = HyperedgeHubRecord {
            id: None,
            kind: kind.to_string(),
            properties,
            source_count,
            target_count,
            parent_hubs: None,
            has_provenance: None,
        };
        let created: Option<HyperedgeHubRecord> =
            self.db.create("hyperedge_hub").content(record).await?;
        let created = created
            .ok_or_else(|| PersistError::InvalidData("failed to create hyperedge_hub".into()))?;
        created
            .id
            .ok_or_else(|| PersistError::InvalidData("created hub has no id".into()))
    }

    pub(crate) async fn relate_source(
        &self,
        node_id: &RecordId,
        hub_id: &RecordId,
        position: usize,
        weight: Option<&str>,
    ) -> Result<(), PersistError> {
        let pos = i64::try_from(position)
            .map_err(|_| PersistError::InvalidData(format!("position overflow: {position}")))?;
        let query = if let Some(w) = weight {
            format!("RELATE $node->source_of->$hub SET position = $pos, weight = <decimal>'{w}'")
        } else {
            "RELATE $node->source_of->$hub SET position = $pos".to_string()
        };
        self.db
            .query(&query)
            .bind(("node", node_id.clone()))
            .bind(("hub", hub_id.clone()))
            .bind(("pos", pos))
            .await?;
        Ok(())
    }

    pub(crate) async fn relate_target(
        &self,
        hub_id: &RecordId,
        node_id: &RecordId,
        position: usize,
        weight: Option<&str>,
    ) -> Result<(), PersistError> {
        let pos = i64::try_from(position)
            .map_err(|_| PersistError::InvalidData(format!("position overflow: {position}")))?;
        let query = if let Some(w) = weight {
            format!("RELATE $hub->target_of->$node SET position = $pos, weight = <decimal>'{w}'")
        } else {
            "RELATE $hub->target_of->$node SET position = $pos".to_string()
        };
        self.db
            .query(&query)
            .bind(("hub", hub_id.clone()))
            .bind(("node", node_id.clone()))
            .bind(("pos", pos))
            .await?;
        Ok(())
    }

    /// Raw source entries with node_id and position, ordered by position.
    pub(crate) async fn source_entries(
        &self,
        hub_id: &RecordId,
    ) -> Result<Vec<ParticipationEntry>, PersistError> {
        let mut result = self
            .db
            .query("SELECT in AS node, position, weight FROM source_of WHERE out = $hub ORDER BY position ASC")
            .bind(("hub", hub_id.clone()))
            .await?;
        let entries: Vec<ParticipationEntry> = result.take(0)?;
        Ok(entries)
    }

    /// Raw target entries with node_id and position, ordered by position.
    pub(crate) async fn target_entries(
        &self,
        hub_id: &RecordId,
    ) -> Result<Vec<ParticipationEntry>, PersistError> {
        let mut result = self
            .db
            .query("SELECT out AS node, position, weight FROM target_of WHERE in = $hub ORDER BY position ASC")
            .bind(("hub", hub_id.clone()))
            .await?;
        let entries: Vec<ParticipationEntry> = result.take(0)?;
        Ok(entries)
    }
}

/// Internal: a participation edge entry (node RecordId + position + optional weight).
///
/// Uses typed `SurrealValue` deserialization to correctly handle `RecordId`
/// (which cannot round-trip through `serde_json::Value`).
#[derive(Debug, serde::Deserialize, surrealdb_types::SurrealValue)]
pub(crate) struct ParticipationEntry {
    pub(crate) node: RecordId,
    #[allow(dead_code)]
    pub(crate) position: i64,
    #[serde(default)]
    pub(crate) weight: Option<String>,
}

/// Format a `RecordId` as a `"table:key"` string.
///
/// `RecordId` does not implement `Display` or `ToString`, so we match
/// on the key variant and produce a stable string that `RecordId::parse_simple`
/// can round-trip.
pub(crate) fn format_record_id(id: &RecordId) -> String {
    use surrealdb::types::RecordIdKey;
    let table = id.table.as_str();
    match &id.key {
        RecordIdKey::String(s) => format!("{table}:{s}"),
        RecordIdKey::Number(n) => format!("{table}:{n}"),
        RecordIdKey::Uuid(u) => format!("{table}:{u}"),
        other => format!("{table}:{other:?}"),
    }
}

/// Extract a Lambda label from a node's properties.
pub(crate) fn extract_label<Lambda: Persistable>(
    node: &GraphNodeRecord,
) -> Result<Lambda, PersistError> {
    let label_val = node
        .properties
        .get("label")
        .ok_or_else(|| PersistError::InvalidData("node missing 'label' property".into()))?;
    Lambda::from_json_value(label_val)
}
