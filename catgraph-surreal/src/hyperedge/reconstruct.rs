use catgraph::cospan::Cospan;
use catgraph::named_cospan::NamedCospan;
use catgraph::span::Span;
use surrealdb::types::RecordId;

use crate::error::PersistError;
use crate::persist::Persistable;
use crate::types_v2::GraphNodeRecord;

use super::{extract_label, HyperedgeStore};

impl HyperedgeStore<'_> {
    /// Get all source nodes for a hub, ordered by position.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::NotFound`] if a source node cannot be fetched.
    /// Returns [`PersistError::Surreal`] on database errors.
    pub async fn sources(&self, hub_id: &RecordId) -> Result<Vec<GraphNodeRecord>, PersistError> {
        let entries = self.source_entries(hub_id).await?;
        let mut nodes = Vec::with_capacity(entries.len());
        for entry in &entries {
            nodes.push(self.node_store.get(&entry.node).await?);
        }
        Ok(nodes)
    }

    /// Get all target nodes for a hub, ordered by position.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::NotFound`] if a target node cannot be fetched.
    /// Returns [`PersistError::Surreal`] on database errors.
    pub async fn targets(&self, hub_id: &RecordId) -> Result<Vec<GraphNodeRecord>, PersistError> {
        let entries = self.target_entries(hub_id).await?;
        let mut nodes = Vec::with_capacity(entries.len());
        for entry in &entries {
            nodes.push(self.node_store.get(&entry.node).await?);
        }
        Ok(nodes)
    }

    /// Reconstruct a `Cospan<Lambda>` from a hub record and its source/target edges.
    ///
    /// Rebuilds the `left_map` and `right_map` by reading `source_of/target_of` positions,
    /// mapping them back to middle node indices.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::InvalidData`] if a source or target node is
    /// not found in the middle set. Returns [`PersistError::NotFound`] if a
    /// node cannot be fetched. Returns [`PersistError::Surreal`] on database errors.
    pub async fn reconstruct_cospan<Lambda: Persistable + Copy>(
        &self,
        hub_id: &RecordId,
    ) -> Result<Cospan<Lambda>, PersistError> {
        // Fetch all middle nodes involved (union of source and target node sets)
        let source_entries = self.source_entries(hub_id).await?;
        let target_entries = self.target_entries(hub_id).await?;

        // Collect unique middle nodes preserving first-seen order
        let mut middle_node_ids: Vec<RecordId> = Vec::new();
        let mut middle_labels: Vec<Lambda> = Vec::new();

        for entry in source_entries.iter().chain(target_entries.iter()) {
            if !middle_node_ids.contains(&entry.node) {
                middle_node_ids.push(entry.node.clone());
                let node = self.node_store.get(&entry.node).await?;
                let label = extract_label::<Lambda>(&node)?;
                middle_labels.push(label);
            }
        }

        // Build left_map: for each source position, find the middle index
        let left_map: Vec<usize> = source_entries
            .iter()
            .map(|e| {
                middle_node_ids
                    .iter()
                    .position(|id| id == &e.node)
                    .ok_or_else(|| PersistError::InvalidData("source node not in middle set".into()))
            })
            .collect::<Result<_, _>>()?;

        // Build right_map: for each target position, find the middle index
        let right_map: Vec<usize> = target_entries
            .iter()
            .map(|e| {
                middle_node_ids
                    .iter()
                    .position(|id| id == &e.node)
                    .ok_or_else(|| PersistError::InvalidData("target node not in middle set".into()))
            })
            .collect::<Result<_, _>>()?;

        Ok(Cospan::new(left_map, right_map, middle_labels))
    }

    /// Reconstruct a `Span<Lambda>` from a hub record and its source/target edges.
    ///
    /// Reads left labels from source entries, right labels from target entries,
    /// and `middle_pairs` from the hub's properties (persisted by `decompose_span`).
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::InvalidData`] if the hub is missing
    /// `middle_pairs` or contains malformed data.
    /// Returns [`PersistError::NotFound`] if a node cannot be fetched.
    /// Returns [`PersistError::Surreal`] on database errors.
    pub async fn reconstruct_span<Lambda: Persistable + Copy>(
        &self,
        hub_id: &RecordId,
    ) -> Result<Span<Lambda>, PersistError> {
        // Fetch source (left) entries ordered by position
        let source_entries = self.source_entries(hub_id).await?;
        let mut left: Vec<Lambda> = Vec::with_capacity(source_entries.len());
        for entry in &source_entries {
            let node = self.node_store.get(&entry.node).await?;
            left.push(extract_label::<Lambda>(&node)?);
        }

        // Fetch target (right) entries ordered by position
        let target_entries = self.target_entries(hub_id).await?;
        let mut right: Vec<Lambda> = Vec::with_capacity(target_entries.len());
        for entry in &target_entries {
            let node = self.node_store.get(&entry.node).await?;
            right.push(extract_label::<Lambda>(&node)?);
        }

        // Read middle_pairs from hub properties
        let hub = self.get_hub(hub_id).await?;
        let pairs_json = hub
            .properties
            .get("middle_pairs")
            .ok_or_else(|| {
                PersistError::InvalidData(
                    "hub missing 'middle_pairs' in properties (not a span hub?)".into(),
                )
            })?;
        let raw_pairs: Vec<[i64; 2]> = serde_json::from_value(pairs_json.clone())?;
        let middle_pairs: Vec<(usize, usize)> = raw_pairs
            .into_iter()
            .map(|[l, r]| {
                let left_idx = usize::try_from(l).map_err(|_| {
                    PersistError::InvalidData(format!("negative left index in middle_pairs: {l}"))
                });
                let right_idx = usize::try_from(r).map_err(|_| {
                    PersistError::InvalidData(format!("negative right index in middle_pairs: {r}"))
                });
                Ok((left_idx?, right_idx?))
            })
            .collect::<Result<_, PersistError>>()?;

        Ok(Span::new(left, right, middle_pairs))
    }

    /// Reconstruct a `NamedCospan<Lambda, String, String>` from a hub record.
    ///
    /// Rebuilds the underlying cospan via [`reconstruct_cospan`](Self::reconstruct_cospan),
    /// then extracts `left_port_names` and `right_port_names` from the hub's
    /// `properties` JSON (persisted by [`decompose_named_cospan`](super::decompose)).
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::InvalidData`] if port name arrays are missing
    /// from the hub properties (e.g. the hub was created via `decompose_cospan`
    /// rather than `decompose_named_cospan`).
    pub async fn reconstruct_named_cospan<Lambda: Persistable + Copy>(
        &self,
        hub_id: &RecordId,
    ) -> Result<NamedCospan<Lambda, String, String>, PersistError> {
        let cospan: Cospan<Lambda> = self.reconstruct_cospan(hub_id).await?;
        let hub = self.get_hub(hub_id).await?;

        let left_port_names: Vec<String> = hub
            .properties
            .get("left_port_names")
            .ok_or_else(|| {
                PersistError::InvalidData(
                    "hub missing 'left_port_names' in properties (not a named cospan hub?)".into(),
                )
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    PersistError::InvalidData(format!("deserialize left_port_names: {e}"))
                })
            })?;

        let right_port_names: Vec<String> = hub
            .properties
            .get("right_port_names")
            .ok_or_else(|| {
                PersistError::InvalidData(
                    "hub missing 'right_port_names' in properties (not a named cospan hub?)".into(),
                )
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    PersistError::InvalidData(format!("deserialize right_port_names: {e}"))
                })
            })?;

        Ok(NamedCospan::new(
            cospan.left_to_middle().to_vec(),
            cospan.right_to_middle().to_vec(),
            cospan.middle().to_vec(),
            left_port_names,
            right_port_names,
        ))
    }
}
