use catgraph::cospan::Cospan;
use catgraph::named_cospan::NamedCospan;
use catgraph::span::Span;
use surrealdb::types::RecordId;

use crate::error::PersistError;
use crate::persist::Persistable;
use crate::types_v2::HyperedgeHubRecord;

use super::HyperedgeStore;

impl<'a> HyperedgeStore<'a> {
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
        let src_count = i64::try_from(left_map.len())
            .map_err(|_| PersistError::InvalidData(format!("source count overflow: {}", left_map.len())))?;
        let tgt_count = i64::try_from(right_map.len())
            .map_err(|_| PersistError::InvalidData(format!("target count overflow: {}", right_map.len())))?;
        let hub_id = self
            .create_hub(hub_kind, hub_properties, src_count, tgt_count)
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
                let l64 = i64::try_from(l)
                    .map_err(|_| PersistError::InvalidData(format!("middle index overflow: {l}")))?;
                let r64 = i64::try_from(r)
                    .map_err(|_| PersistError::InvalidData(format!("middle index overflow: {r}")))?;
                Ok([l64, r64])
            })
            .collect::<Result<Vec<_>, PersistError>>()?;
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
        let src_count = i64::try_from(left.len())
            .map_err(|_| PersistError::InvalidData(format!("source count overflow: {}", left.len())))?;
        let tgt_count = i64::try_from(right.len())
            .map_err(|_| PersistError::InvalidData(format!("target count overflow: {}", right.len())))?;
        let hub_id = self
            .create_hub(hub_kind, props, src_count, tgt_count)
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
        let src_count = i64::try_from(left_map.len())
            .map_err(|_| PersistError::InvalidData(format!("source count overflow: {}", left_map.len())))?;
        let tgt_count = i64::try_from(right_map.len())
            .map_err(|_| PersistError::InvalidData(format!("target count overflow: {}", right_map.len())))?;

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
}
