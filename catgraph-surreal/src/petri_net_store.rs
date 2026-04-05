use std::collections::HashMap;

use catgraph::petri_net::{Marking, PetriNet, Transition};
use rust_decimal::Decimal;
use surrealdb::engine::local::Db;
use surrealdb::types::RecordId;
use surrealdb::Surreal;
use surrealdb_types::SurrealValue;

use crate::error::PersistError;
use crate::persist::Persistable;
use crate::types_v2::{MarkingRecord, PetriNetRecord, PetriPlaceRecord, PetriTransitionRecord};

/// Store for `PetriNet<Lambda>` persistence in SurrealDB.
///
/// Decomposes a Petri net into first-class `petri_net`, `petri_place`,
/// `petri_transition`, `pre_arc`, and `post_arc` records.
/// Markings are stored as separate `petri_marking` snapshots.
pub struct PetriNetStore<'a> {
    db: &'a Surreal<Db>,
}

/// Helper for deserializing arc query results.
///
/// SurrealDB `in` is a reserved word, so we alias it as `src` in pre-arc
/// queries and `dst` in post-arc queries. The `SurrealValue` derive does
/// not support `#[serde(rename)]`, so we use aliases matching the query.
///
/// Weight is cast to string in the query (`<string>weight`) because SurrealDB's
/// `decimal` type deserializes as a number, not a string.
#[derive(Debug, serde::Deserialize, SurrealValue)]
struct PreArcEntry {
    src: RecordId,
    weight: String,
}

#[derive(Debug, serde::Deserialize, SurrealValue)]
struct PostArcEntry {
    dst: RecordId,
    weight: String,
}

impl<'a> PetriNetStore<'a> {
    pub fn new(db: &'a Surreal<Db>) -> Self {
        Self { db }
    }

    /// Save a `PetriNet<Lambda>` to SurrealDB, returning the net's `RecordId`.
    pub async fn save<Lambda: Persistable + Copy>(
        &self,
        net: &PetriNet<Lambda>,
        name: &str,
    ) -> Result<RecordId, PersistError> {
        // 1. Create the petri_net record
        let net_record = PetriNetRecord {
            id: None,
            name: name.to_string(),
            label_type: Lambda::type_name().to_string(),
            properties: serde_json::json!({}),
        };
        let created: Option<PetriNetRecord> =
            self.db.create("petri_net").content(net_record).await?;
        let created = created
            .ok_or_else(|| PersistError::InvalidData("failed to create petri_net record".into()))?;
        let net_id = created
            .id
            .ok_or_else(|| PersistError::InvalidData("created petri_net has no id".into()))?;

        // 2. Create place records
        let mut place_ids: Vec<RecordId> = Vec::with_capacity(net.place_count());
        for (i, place_label) in net.places().iter().enumerate() {
            let pos = i64::try_from(i)
                .map_err(|_| PersistError::InvalidData(format!("position overflow: {i}")))?;
            let place_record = PetriPlaceRecord {
                id: None,
                net: net_id.clone(),
                position: pos,
                label: place_label.to_json_value().to_string(),
                label_type: Lambda::type_name().to_string(),
                properties: serde_json::json!({}),
            };
            let created: Option<PetriPlaceRecord> =
                self.db.create("petri_place").content(place_record).await?;
            let created = created.ok_or_else(|| {
                PersistError::InvalidData("failed to create petri_place record".into())
            })?;
            let place_id = created
                .id
                .ok_or_else(|| PersistError::InvalidData("created place has no id".into()))?;
            place_ids.push(place_id);
        }

        // 3. Create transition records and arcs
        for (t_idx, transition) in net.transitions().iter().enumerate() {
            let t_pos = i64::try_from(t_idx)
                .map_err(|_| PersistError::InvalidData(format!("position overflow: {t_idx}")))?;
            let trans_record = PetriTransitionRecord {
                id: None,
                net: net_id.clone(),
                position: t_pos,
                properties: serde_json::json!({}),
            };
            let created: Option<PetriTransitionRecord> = self
                .db
                .create("petri_transition")
                .content(trans_record)
                .await?;
            let created = created.ok_or_else(|| {
                PersistError::InvalidData("failed to create petri_transition record".into())
            })?;
            let trans_id = created
                .id
                .ok_or_else(|| PersistError::InvalidData("created transition has no id".into()))?;

            // 4. Pre-arcs: place -> transition
            for (place_idx, weight) in transition.pre() {
                let place_id = &place_ids[*place_idx];
                let query = format!(
                    "RELATE $place->pre_arc->$trans SET weight = <decimal>'{weight}'"
                );
                self.db
                    .query(&query)
                    .bind(("place", place_id.clone()))
                    .bind(("trans", trans_id.clone()))
                    .await?;
            }

            // 5. Post-arcs: transition -> place
            for (place_idx, weight) in transition.post() {
                let place_id = &place_ids[*place_idx];
                let query = format!(
                    "RELATE $trans->post_arc->$place SET weight = <decimal>'{weight}'"
                );
                self.db
                    .query(&query)
                    .bind(("trans", trans_id.clone()))
                    .bind(("place", place_id.clone()))
                    .await?;
            }
        }

        Ok(net_id)
    }

    /// Load a `PetriNet<Lambda>` from SurrealDB by its net `RecordId`.
    pub async fn load<Lambda: Persistable + Copy>(
        &self,
        net_id: &RecordId,
    ) -> Result<PetriNet<Lambda>, PersistError> {
        // 1. Fetch the net record and verify label_type
        let net_record: Option<PetriNetRecord> = self.db.select(net_id).await?;
        let net_record =
            net_record.ok_or_else(|| PersistError::NotFound(format!("{net_id:?}")))?;
        if net_record.label_type != Lambda::type_name() {
            return Err(PersistError::TypeMismatch {
                expected: Lambda::type_name().into(),
                got: net_record.label_type,
            });
        }

        // 2. Fetch places ordered by position
        let mut result = self
            .db
            .query("SELECT * FROM petri_place WHERE net = $net ORDER BY position ASC")
            .bind(("net", net_id.clone()))
            .await?;
        let place_records: Vec<PetriPlaceRecord> = result.take(0)?;

        // Build places vector and place_id -> index map
        let mut places: Vec<Lambda> = Vec::with_capacity(place_records.len());
        let mut place_id_to_idx: HashMap<String, usize> = HashMap::new();
        for (i, pr) in place_records.iter().enumerate() {
            let label_val: serde_json::Value = serde_json::from_str(&pr.label)
                .map_err(|e| PersistError::InvalidData(e.to_string()))?;
            let label = Lambda::from_json_value(&label_val)?;
            places.push(label);
            if let Some(ref id) = pr.id {
                place_id_to_idx.insert(format_record_id(id), i);
            }
        }

        // 3. Fetch transitions ordered by position
        let mut result = self
            .db
            .query("SELECT * FROM petri_transition WHERE net = $net ORDER BY position ASC")
            .bind(("net", net_id.clone()))
            .await?;
        let trans_records: Vec<PetriTransitionRecord> = result.take(0)?;

        // 4. For each transition, fetch pre-arcs and post-arcs
        let mut transitions: Vec<Transition> = Vec::with_capacity(trans_records.len());
        for tr in &trans_records {
            let trans_id = tr
                .id
                .as_ref()
                .ok_or_else(|| PersistError::InvalidData("transition has no id".into()))?;

            // Pre-arcs: cast weight to string for correct deserialization
            let mut result = self
                .db
                .query("SELECT `in` AS src, <string>weight AS weight FROM pre_arc WHERE out = $trans")
                .bind(("trans", trans_id.clone()))
                .await?;
            let pre_entries: Vec<PreArcEntry> = result.take(0)?;

            let mut pre: Vec<(usize, Decimal)> = Vec::with_capacity(pre_entries.len());
            for entry in &pre_entries {
                let idx = place_id_to_idx
                    .get(&format_record_id(&entry.src))
                    .ok_or_else(|| {
                        PersistError::InvalidData(format!(
                            "pre-arc references unknown place {:?}",
                            entry.src
                        ))
                    })?;
                let weight: Decimal = entry.weight.parse().map_err(|e| {
                    PersistError::InvalidData(format!("invalid decimal weight: {e}"))
                })?;
                pre.push((*idx, weight));
            }

            // Post-arcs: cast weight to string for correct deserialization
            let mut result = self
                .db
                .query("SELECT out AS dst, <string>weight AS weight FROM post_arc WHERE `in` = $trans")
                .bind(("trans", trans_id.clone()))
                .await?;
            let post_entries: Vec<PostArcEntry> = result.take(0)?;

            let mut post: Vec<(usize, Decimal)> = Vec::with_capacity(post_entries.len());
            for entry in &post_entries {
                let idx = place_id_to_idx
                    .get(&format_record_id(&entry.dst))
                    .ok_or_else(|| {
                        PersistError::InvalidData(format!(
                            "post-arc references unknown place {:?}",
                            entry.dst
                        ))
                    })?;
                let weight: Decimal = entry.weight.parse().map_err(|e| {
                    PersistError::InvalidData(format!("invalid decimal weight: {e}"))
                })?;
                post.push((*idx, weight));
            }

            // Sort arcs by place index for deterministic ordering
            pre.sort_by_key(|(idx, _)| *idx);
            post.sort_by_key(|(idx, _)| *idx);
            transitions.push(Transition::new(pre, post));
        }

        Ok(PetriNet::new(places, transitions))
    }

    /// Save a marking snapshot for a Petri net.
    pub async fn save_marking(
        &self,
        net_id: &RecordId,
        marking: &Marking,
        label: &str,
    ) -> Result<RecordId, PersistError> {
        // Serialize tokens as JSON object: {"place_idx": "decimal_value", ...}
        let mut tokens_map = serde_json::Map::new();
        for (place_idx, count) in marking.tokens() {
            if !count.is_zero() {
                tokens_map.insert(place_idx.to_string(), serde_json::json!(count.to_string()));
            }
        }

        let record = MarkingRecord {
            id: None,
            net: net_id.clone(),
            label: label.to_string(),
            tokens: serde_json::Value::Object(tokens_map),
            step: None,
        };
        let created: Option<MarkingRecord> =
            self.db.create("petri_marking").content(record).await?;
        let created = created.ok_or_else(|| {
            PersistError::InvalidData("failed to create petri_marking record".into())
        })?;
        created
            .id
            .ok_or_else(|| PersistError::InvalidData("created marking has no id".into()))
    }

    /// Load a marking snapshot by its `RecordId`.
    pub async fn load_marking(
        &self,
        marking_id: &RecordId,
    ) -> Result<Marking, PersistError> {
        let record: Option<MarkingRecord> = self.db.select(marking_id).await?;
        let record =
            record.ok_or_else(|| PersistError::NotFound(format!("{marking_id:?}")))?;

        let pairs = parse_tokens_object(&record.tokens)?;
        Ok(Marking::from_vec(pairs))
    }

    /// Delete a Petri net and all its related records.
    ///
    /// Deletes in dependency order: arcs, transitions, places, markings, then the net itself.
    pub async fn delete(&self, net_id: &RecordId) -> Result<(), PersistError> {
        // Delete pre-arcs referencing places in this net
        self.db
            .query("DELETE pre_arc WHERE `in`.net = $net OR out.net = $net")
            .bind(("net", net_id.clone()))
            .await?;

        // Delete post-arcs referencing transitions in this net
        self.db
            .query("DELETE post_arc WHERE `in`.net = $net OR out.net = $net")
            .bind(("net", net_id.clone()))
            .await?;

        // Delete transitions
        self.db
            .query("DELETE petri_transition WHERE net = $net")
            .bind(("net", net_id.clone()))
            .await?;

        // Delete places
        self.db
            .query("DELETE petri_place WHERE net = $net")
            .bind(("net", net_id.clone()))
            .await?;

        // Delete markings
        self.db
            .query("DELETE petri_marking WHERE net = $net")
            .bind(("net", net_id.clone()))
            .await?;

        // Delete the net itself
        let _: Option<PetriNetRecord> = self.db.delete(net_id).await?;
        Ok(())
    }

    /// List all Petri net records.
    pub async fn list(&self) -> Result<Vec<PetriNetRecord>, PersistError> {
        let records: Vec<PetriNetRecord> = self.db.select("petri_net").await?;
        Ok(records)
    }
}

/// Format a `RecordId` as a `"table:key"` string for use as a map key.
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

/// Parse the tokens JSON object into `Vec<(usize, Decimal)>` pairs.
fn parse_tokens_object(
    tokens: &serde_json::Value,
) -> Result<Vec<(usize, Decimal)>, PersistError> {
    let obj = tokens
        .as_object()
        .ok_or_else(|| PersistError::InvalidData("tokens is not an object".into()))?;
    let mut pairs = Vec::with_capacity(obj.len());
    for (key, val) in obj {
        let idx: usize = key.parse().map_err(|e| {
            PersistError::InvalidData(format!("invalid place index '{key}': {e}"))
        })?;
        let val_str = val.as_str().ok_or_else(|| {
            PersistError::InvalidData(format!("token value for place {key} is not a string"))
        })?;
        let count: Decimal = val_str.parse().map_err(|e| {
            PersistError::InvalidData(format!("invalid decimal token count '{val_str}': {e}"))
        })?;
        if !count.is_zero() {
            pairs.push((idx, count));
        }
    }
    Ok(pairs)
}
