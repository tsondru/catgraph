use catgraph::cospan::Cospan;
use surrealdb::types::RecordId;

use crate::error::PersistError;
use crate::persist::Persistable;
use crate::types_v2::{ComposedFromRecord, HyperedgeHubRecord};

use crate::utils::format_record_id;

use super::HyperedgeStore;

impl<'a> HyperedgeStore<'a> {
    /// Decompose a cospan with composition provenance tracking.
    ///
    /// Wraps [`decompose_cospan`](Self::decompose_cospan) but injects `parent_hubs`
    /// into the hub's `properties` JSON object so that the lineage of composed
    /// hubs can be queried later.
    ///
    /// `parent_hub_ids` records which existing hubs were composed to produce this
    /// cospan. They are stored as `"table:key"` strings because `RecordId` cannot
    /// round-trip through `serde_json::Value`.
    pub async fn decompose_cospan_with_provenance<Lambda, F>(
        &self,
        cospan: &Cospan<Lambda>,
        hub_kind: &str,
        hub_properties: serde_json::Value,
        node_namer: F,
        parent_hub_ids: &[RecordId],
    ) -> Result<RecordId, PersistError>
    where
        Lambda: Persistable + Copy,
        F: Fn(&Lambda) -> String,
    {
        let mut props = hub_properties;
        if let Some(obj) = props.as_object_mut() {
            let parent_ids: Vec<String> =
                parent_hub_ids.iter().map(format_record_id).collect();
            obj.insert("parent_hubs".into(), serde_json::json!(parent_ids));
        }
        let hub_id = self
            .decompose_cospan(cospan, hub_kind, props, node_namer)
            .await?;

        // Set schema-level REFERENCE parent_hubs field (RecordId array)
        if !parent_hub_ids.is_empty() {
            self.db
                .query("UPDATE $hub_id SET parent_hubs = $parents")
                .bind(("hub_id", hub_id.clone()))
                .bind(("parents", parent_hub_ids.to_vec()))
                .await
                .map_err(PersistError::Surreal)?;
        }

        Ok(hub_id)
    }

    /// Get the parent hub IDs from a hub's provenance metadata.
    ///
    /// Returns an empty `Vec` if no provenance was recorded (i.e. the hub was
    /// created via plain [`decompose_cospan`](Self::decompose_cospan)).
    pub async fn composition_parents(
        &self,
        hub_id: &RecordId,
    ) -> Result<Vec<String>, PersistError> {
        let hub = self.get_hub(hub_id).await?;
        match hub.properties.get("parent_hubs") {
            Some(parents) => {
                let ids: Vec<String> = serde_json::from_value(parents.clone())?;
                Ok(ids)
            }
            None => Ok(vec![]),
        }
    }

    /// Find all hubs that were composed from a given parent hub.
    ///
    /// Searches `properties.parent_hubs` arrays across all `hyperedge_hub` records
    /// for the string representation of `hub_id`.
    pub async fn composition_children(
        &self,
        hub_id: &RecordId,
    ) -> Result<Vec<HyperedgeHubRecord>, PersistError> {
        let hub_str = format_record_id(hub_id);
        let mut result = self
            .db
            .query(
                "SELECT * FROM hyperedge_hub \
                 WHERE properties.parent_hubs CONTAINS $parent_id",
            )
            .bind(("parent_id", hub_str))
            .await
            .map_err(PersistError::Surreal)?;
        let hubs: Vec<HyperedgeHubRecord> = result.take(0).map_err(PersistError::Surreal)?;
        Ok(hubs)
    }

    /// Create a `composed_from` RELATE edge between parent and child hubs.
    pub async fn relate_composition(
        &self,
        parent_hub_id: &RecordId,
        child_hub_id: &RecordId,
        operation: &str,
    ) -> Result<RecordId, PersistError> {
        let mut result = self
            .db
            .query("RELATE $parent->composed_from->$child SET operation = $op")
            .bind(("parent", parent_hub_id.clone()))
            .bind(("child", child_hub_id.clone()))
            .bind(("op", operation.to_owned()))
            .await
            .map_err(PersistError::Surreal)?;
        let relation: Option<ComposedFromRecord> =
            result.take(0).map_err(PersistError::Surreal)?;
        relation
            .and_then(|r| r.id)
            .ok_or_else(|| PersistError::InvalidData("Failed to create composition relation".into()))
    }

    /// Find child hubs via the schema-level `parent_hubs` REFERENCE field.
    ///
    /// Returns all hubs whose `parent_hubs` array contains the given hub ID.
    pub async fn composed_children_via_ref(
        &self,
        hub_id: &RecordId,
    ) -> Result<Vec<HyperedgeHubRecord>, PersistError> {
        let mut result = self
            .db
            .query("SELECT * FROM hyperedge_hub WHERE parent_hubs CONTAINS $hub_id")
            .bind(("hub_id", hub_id.clone()))
            .await
            .map_err(PersistError::Surreal)?;
        let hubs: Vec<HyperedgeHubRecord> = result.take(0).map_err(PersistError::Surreal)?;
        Ok(hubs)
    }
}
