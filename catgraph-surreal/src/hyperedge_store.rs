use catgraph::cospan::Cospan;
use catgraph::named_cospan::NamedCospan;
use catgraph::span::Span;
use surrealdb::engine::local::Db;
use surrealdb::types::RecordId;
use surrealdb::Surreal;
use surrealdb_types::SurrealValue;

use crate::error::PersistError;
use crate::node_store::NodeStore;
use crate::persist::Persistable;
use crate::types_v2::{ComposedFromRecord, GraphNodeRecord, HyperedgeHubRecord};

/// Store for n-ary hyperedges using hub-node reification in the V2 schema.
///
/// Decomposes catgraph's `Cospan` and `Span` types into:
/// - A hub record (`hyperedge_hub`) representing the hyperedge
/// - `source_of` RELATE edges from source nodes to the hub
/// - `target_of` RELATE edges from the hub to target nodes
pub struct HyperedgeStore<'a> {
    db: &'a Surreal<Db>,
    node_store: NodeStore<'a>,
}

impl<'a> HyperedgeStore<'a> {
    pub fn new(db: &'a Surreal<Db>) -> Self {
        Self {
            db,
            node_store: NodeStore::new(db),
        }
    }

    /// Decompose a `Cospan<Lambda>` into V2 graph records.
    ///
    /// The cospan `left --left_map--> middle <--right_map-- right` becomes:
    /// 1. One `graph_node` per middle element (labelled with Lambda value)
    /// 2. One `hyperedge_hub` record
    /// 3. `source_of` edges: for each left index i, RELATE middle[left_map[i]] -> hub (position=i)
    /// 4. `target_of` edges: for each right index j, RELATE hub -> middle[right_map[j]] (position=j)
    ///
    /// `node_namer` converts a Lambda label to a node name string.
    pub async fn decompose_cospan<Lambda, F>(
        &self,
        cospan: &Cospan<Lambda>,
        hub_kind: &str,
        hub_properties: serde_json::Value,
        node_namer: F,
    ) -> Result<RecordId, PersistError>
    where
        Lambda: Persistable + Copy,
        F: Fn(&Lambda) -> String,
    {
        let middle = cospan.middle();
        let left_map = cospan.left_to_middle();
        let right_map = cospan.right_to_middle();

        // Create hub
        let hub_id = self
            .create_hub(
                hub_kind,
                hub_properties,
                i64::try_from(left_map.len()).unwrap_or(0),
                i64::try_from(right_map.len()).unwrap_or(0),
            )
            .await?;

        // Create middle nodes
        let mut middle_node_ids = Vec::with_capacity(middle.len());
        for label in middle {
            let name = node_namer(label);
            let label_json = label.to_json_value();
            let props = serde_json::json!({ "label": label_json, "label_type": Lambda::type_name() });
            let node_id = self
                .node_store
                .create(&name, "middle", vec![], props)
                .await?;
            middle_node_ids.push(node_id);
        }

        // RELATE sources: left[i] maps to middle[left_map[i]]
        for (pos, &mid_idx) in left_map.iter().enumerate() {
            self.relate_source(&middle_node_ids[mid_idx], &hub_id, pos)
                .await?;
        }

        // RELATE targets: right[j] maps to middle[right_map[j]]
        for (pos, &mid_idx) in right_map.iter().enumerate() {
            self.relate_target(&hub_id, &middle_node_ids[mid_idx], pos)
                .await?;
        }

        Ok(hub_id)
    }

    /// Decompose a `Span<Lambda>` into V2 graph records.
    ///
    /// The span `left <--middle_pairs--> right` becomes:
    /// 1. One `graph_node` per left element + one per right element
    /// 2. One `hyperedge_hub` record
    /// 3. `source_of` edges from left nodes to hub (by position)
    /// 4. `target_of` edges from hub to right nodes (by position)
    pub async fn decompose_span<Lambda, F>(
        &self,
        span: &Span<Lambda>,
        hub_kind: &str,
        hub_properties: serde_json::Value,
        node_namer: F,
    ) -> Result<RecordId, PersistError>
    where
        Lambda: Persistable + Copy,
        F: Fn(&Lambda) -> String,
    {
        let left = span.left();
        let right = span.right();

        // Inject middle_pairs and identity flags into hub properties
        let pairs: Vec<[i64; 2]> = span
            .middle_pairs()
            .iter()
            .map(|&(l, r)| {
                [
                    i64::try_from(l).unwrap_or(0),
                    i64::try_from(r).unwrap_or(0),
                ]
            })
            .collect();
        let mut props = hub_properties;
        if let Some(obj) = props.as_object_mut() {
            obj.insert("middle_pairs".into(), serde_json::json!(pairs));
            obj.insert(
                "is_left_id".into(),
                serde_json::json!(span.is_left_identity()),
            );
            obj.insert(
                "is_right_id".into(),
                serde_json::json!(span.is_right_identity()),
            );
        }

        // Create hub
        let hub_id = self
            .create_hub(
                hub_kind,
                props,
                i64::try_from(left.len()).unwrap_or(0),
                i64::try_from(right.len()).unwrap_or(0),
            )
            .await?;

        // Create left nodes
        let mut left_node_ids = Vec::with_capacity(left.len());
        for (pos, label) in left.iter().enumerate() {
            let name = node_namer(label);
            let props = serde_json::json!({ "label": label.to_json_value(), "label_type": Lambda::type_name() });
            let node_id = self
                .node_store
                .create(&name, "source", vec![], props)
                .await?;
            self.relate_source(&node_id, &hub_id, pos).await?;
            left_node_ids.push(node_id);
        }

        // Create right nodes
        let mut right_node_ids = Vec::with_capacity(right.len());
        for (pos, label) in right.iter().enumerate() {
            let name = node_namer(label);
            let props = serde_json::json!({ "label": label.to_json_value(), "label_type": Lambda::type_name() });
            let node_id = self
                .node_store
                .create(&name, "target", vec![], props)
                .await?;
            self.relate_target(&hub_id, &node_id, pos).await?;
            right_node_ids.push(node_id);
        }

        Ok(hub_id)
    }

    /// Decompose a `NamedCospan` into V2 graph records.
    ///
    /// Like `decompose_cospan` but uses port names as node names.
    pub async fn decompose_named_cospan<Lambda>(
        &self,
        nc: &NamedCospan<Lambda, String, String>,
        hub_kind: &str,
        hub_properties: serde_json::Value,
    ) -> Result<RecordId, PersistError>
    where
        Lambda: Persistable + Copy,
    {
        self.decompose_cospan(nc.cospan(), hub_kind, hub_properties, |l| {
            l.to_json_value().to_string()
        })
        .await
    }

    /// Decompose a `Cospan<Lambda>` atomically — all records created in a single transaction.
    ///
    /// Unlike [`decompose_cospan`](Self::decompose_cospan) which issues separate CREATE/RELATE
    /// calls (any of which could fail leaving orphaned records), this method builds a single
    /// multi-statement SurrealQL query wrapped in `BEGIN TRANSACTION ... COMMIT TRANSACTION`.
    ///
    /// Within the transaction, `LET` variables capture each created record so that
    /// subsequent `RELATE` statements can reference them by variable name.
    ///
    /// On success, returns the `RecordId` of the created hub.
    pub async fn decompose_cospan_atomic<Lambda, F>(
        &self,
        cospan: &Cospan<Lambda>,
        hub_kind: &str,
        hub_properties: serde_json::Value,
        node_namer: F,
    ) -> Result<RecordId, PersistError>
    where
        Lambda: Persistable + Copy,
        F: Fn(&Lambda) -> String,
    {
        let middle = cospan.middle();
        let left_map = cospan.left_to_middle();
        let right_map = cospan.right_to_middle();
        let src_count = i64::try_from(left_map.len()).unwrap_or(0);
        let tgt_count = i64::try_from(right_map.len()).unwrap_or(0);

        // Build the transaction query string.
        // LET variables are scoped to the transaction and available across statements.
        let mut query = String::from("BEGIN TRANSACTION;\n");

        // 1. Create hub
        query.push_str(&format!(
            "LET $hub = CREATE ONLY hyperedge_hub CONTENT {{\
             kind: $hub_kind, properties: $hub_props, \
             source_count: {src_count}, target_count: {tgt_count} }};\n"
        ));

        // 2. Create middle nodes (one per unique middle element)
        for i in 0..middle.len() {
            query.push_str(&format!(
                "LET $node_{i} = CREATE ONLY graph_node CONTENT {{\
                 name: $name_{i}, kind: 'middle', labels: [], \
                 properties: {{ label: $label_{i}, label_type: $ltype }} }};\n"
            ));
        }

        // 3. RELATE source_of edges (node -> hub, with position)
        for (pos, &mid_idx) in left_map.iter().enumerate() {
            query.push_str(&format!(
                "RELATE $node_{mid_idx}->source_of->$hub SET position = {pos};\n"
            ));
        }

        // 4. RELATE target_of edges (hub -> node, with position)
        for (pos, &mid_idx) in right_map.iter().enumerate() {
            query.push_str(&format!(
                "RELATE $hub->target_of->$node_{mid_idx} SET position = {pos};\n"
            ));
        }

        // RETURN the hub record before COMMIT so we can extract it.
        query.push_str("RETURN $hub;\n");
        query.push_str("COMMIT TRANSACTION;\n");

        // Bind parameters.
        let mut builder = self
            .db
            .query(&query)
            .bind(("hub_kind", hub_kind.to_string()))
            .bind(("hub_props", hub_properties))
            .bind(("ltype", Lambda::type_name().to_string()));

        for (i, label) in middle.iter().enumerate() {
            let name = node_namer(label);
            let label_json = label.to_json_value();
            builder = builder
                .bind((format!("name_{i}"), name))
                .bind((format!("label_{i}"), label_json));
        }

        let mut result = builder.await.map_err(PersistError::Surreal)?;

        // Each statement in the transaction occupies one result index:
        //   0: BEGIN TRANSACTION
        //   1: LET $hub = CREATE ...
        //   2..2+N-1: LET $node_i = CREATE ...  (N = middle.len())
        //   2+N..2+N+M-1: RELATE source_of      (M = left_map.len())
        //   2+N+M..2+N+M+K-1: RELATE target_of  (K = right_map.len())
        //   2+N+M+K: RETURN $hub                 <-- this is what we want
        //   2+N+M+K+1: COMMIT TRANSACTION
        let return_idx = 2 + middle.len() + left_map.len() + right_map.len();
        let hub_record: Option<HyperedgeHubRecord> =
            result.take(return_idx).map_err(PersistError::Surreal)?;

        let hub = hub_record.ok_or_else(|| {
            PersistError::InvalidData(
                "atomic decompose: transaction returned no hub record".into(),
            )
        })?;
        hub.id.ok_or_else(|| {
            PersistError::InvalidData("atomic decompose: created hub has no id".into())
        })
    }

    /// Decompose a cospan atomically with retry on `TransactionConflict`.
    ///
    /// Uses exponential backoff starting at 50ms, doubling each attempt.
    /// Useful when multiple concurrent writers may conflict on the same records.
    pub async fn decompose_cospan_with_retry<Lambda, F>(
        &self,
        cospan: &Cospan<Lambda>,
        hub_kind: &str,
        hub_properties: serde_json::Value,
        node_namer: F,
        max_retries: u32,
    ) -> Result<RecordId, PersistError>
    where
        Lambda: Persistable + Copy,
        F: Fn(&Lambda) -> String + Clone,
    {
        let base_delay = std::time::Duration::from_millis(50);
        for attempt in 0..=max_retries {
            match self
                .decompose_cospan_atomic(
                    cospan,
                    hub_kind,
                    hub_properties.clone(),
                    node_namer.clone(),
                )
                .await
            {
                Ok(id) => return Ok(id),
                Err(e) if e.is_transaction_conflict() && attempt < max_retries => {
                    tokio::time::sleep(base_delay * 2u32.pow(attempt)).await;
                }
                Err(e) => return Err(e),
            }
        }
        unreachable!()
    }

    /// Get all source nodes for a hub, ordered by position.
    pub async fn sources(&self, hub_id: &RecordId) -> Result<Vec<GraphNodeRecord>, PersistError> {
        let entries = self.source_entries(hub_id).await?;
        let mut nodes = Vec::with_capacity(entries.len());
        for entry in &entries {
            nodes.push(self.node_store.get(&entry.node).await?);
        }
        Ok(nodes)
    }

    /// Get all target nodes for a hub, ordered by position.
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
    /// Rebuilds the left_map and right_map by reading source_of/target_of positions,
    /// mapping them back to middle node indices.
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

    // --- composition provenance ---

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

    // --- record reference & composition relation methods ---

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

    // --- internal helpers ---

    async fn create_hub(
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

    async fn relate_source(
        &self,
        node_id: &RecordId,
        hub_id: &RecordId,
        position: usize,
    ) -> Result<(), PersistError> {
        let pos = i64::try_from(position)
            .map_err(|_| PersistError::InvalidData(format!("position overflow: {position}")))?;
        self.db
            .query("RELATE $node->source_of->$hub SET position = $pos")
            .bind(("node", node_id.clone()))
            .bind(("hub", hub_id.clone()))
            .bind(("pos", pos))
            .await?;
        Ok(())
    }

    async fn relate_target(
        &self,
        hub_id: &RecordId,
        node_id: &RecordId,
        position: usize,
    ) -> Result<(), PersistError> {
        let pos = i64::try_from(position)
            .map_err(|_| PersistError::InvalidData(format!("position overflow: {position}")))?;
        self.db
            .query("RELATE $hub->target_of->$node SET position = $pos")
            .bind(("hub", hub_id.clone()))
            .bind(("node", node_id.clone()))
            .bind(("pos", pos))
            .await?;
        Ok(())
    }

    /// Raw source entries with node_id and position, ordered by position.
    async fn source_entries(
        &self,
        hub_id: &RecordId,
    ) -> Result<Vec<ParticipationEntry>, PersistError> {
        let mut result = self
            .db
            .query("SELECT in AS node, position FROM source_of WHERE out = $hub ORDER BY position ASC")
            .bind(("hub", hub_id.clone()))
            .await?;
        let entries: Vec<ParticipationEntry> = result.take(0)?;
        Ok(entries)
    }

    /// Raw target entries with node_id and position, ordered by position.
    async fn target_entries(
        &self,
        hub_id: &RecordId,
    ) -> Result<Vec<ParticipationEntry>, PersistError> {
        let mut result = self
            .db
            .query("SELECT out AS node, position FROM target_of WHERE in = $hub ORDER BY position ASC")
            .bind(("hub", hub_id.clone()))
            .await?;
        let entries: Vec<ParticipationEntry> = result.take(0)?;
        Ok(entries)
    }
}

/// Internal: a participation edge entry (node RecordId + position).
///
/// Uses typed `SurrealValue` deserialization to correctly handle `RecordId`
/// (which cannot round-trip through `serde_json::Value`).
#[derive(Debug, serde::Deserialize, surrealdb_types::SurrealValue)]
struct ParticipationEntry {
    node: RecordId,
    #[allow(dead_code)]
    position: i64,
}

/// Format a `RecordId` as a `"table:key"` string.
///
/// `RecordId` does not implement `Display` or `ToString`, so we match
/// on the key variant and produce a stable string that `RecordId::parse_simple`
/// can round-trip.
fn format_record_id(id: &RecordId) -> String {
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
fn extract_label<Lambda: Persistable>(node: &GraphNodeRecord) -> Result<Lambda, PersistError> {
    let label_val = node
        .properties
        .get("label")
        .ok_or_else(|| PersistError::InvalidData("node missing 'label' property".into()))?;
    Lambda::from_json_value(label_val)
}
