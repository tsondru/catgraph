//! Persistence layer for [`PetriNet<Lambda>`] in SurrealDB.
//!
//! Decomposes a Petri net into first-class records: `petri_net` (the net itself),
//! `petri_place` (typed places), `petri_transition` (transitions), plus `pre_arc`
//! and `post_arc` RELATE edges encoding the bipartite incidence structure.
//! Markings are stored as separate `petri_marking` snapshots with token counts
//! serialized as `{"place_index": "decimal_value"}` JSON objects.
//!
//! Arc weights use SurrealDB's `decimal` type to preserve exact [`Decimal`] values.

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
use crate::utils::format_record_id;

/// Async CRUD store for [`PetriNet<Lambda>`] persistence in SurrealDB.
///
/// Each Petri net is decomposed into a `petri_net` header record, one
/// `petri_place` record per place (ordered by position), one
/// `petri_transition` record per transition, and RELATE-based `pre_arc` /
/// `post_arc` edges encoding the bipartite incidence between places and
/// transitions. [`Marking`] snapshots are stored independently and linked
/// back to their parent net via a `net` foreign key.
///
/// Reconstruction (`load`) re-sorts arcs by place index to guarantee
/// deterministic ordering regardless of SurrealDB query return order.
pub struct PetriNetStore<'a> {
    /// Borrowed database connection used for all queries.
    db: &'a Surreal<Db>,
}

/// Deserialization helper for pre-arc query results.
///
/// SurrealDB's `in` is a reserved keyword, so the query aliases it as `src`.
/// The `SurrealValue` derive does not support `#[serde(rename)]`, hence the
/// alias must match the struct field name exactly.
///
/// Weight is cast to string in the query (`<string>weight`) because SurrealDB's
/// `decimal` type deserializes as a number rather than a string, which would
/// lose precision for [`Decimal`] parsing.
#[derive(Debug, serde::Deserialize, SurrealValue)]
struct PreArcEntry {
    /// Source place `RecordId` (aliased from `in`).
    src: RecordId,
    /// Arc weight as a string-cast decimal for lossless [`Decimal`] parsing.
    weight: String,
}

/// Deserialization helper for post-arc query results.
///
/// Mirrors [`PreArcEntry`] but for the outbound direction: the query aliases
/// `out` as `dst`.
#[derive(Debug, serde::Deserialize, SurrealValue)]
struct PostArcEntry {
    /// Target place `RecordId` (aliased from `out`).
    dst: RecordId,
    /// Arc weight as a string-cast decimal for lossless [`Decimal`] parsing.
    weight: String,
}

impl<'a> PetriNetStore<'a> {
    pub fn new(db: &'a Surreal<Db>) -> Self {
        Self { db }
    }

    /// Save a [`PetriNet<Lambda>`] to SurrealDB, returning the net's [`RecordId`].
    ///
    /// Creates records in dependency order: the net header, then places
    /// (preserving index order via a `position` field), then transitions
    /// with their pre-arc and post-arc RELATE edges. Arc weights are stored
    /// as SurrealDB `decimal` values for exact representation.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::InvalidData`] if record creation fails or a
    /// position index overflows `i64`. Returns [`PersistError::Surreal`] on
    /// database communication errors.
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

    /// Load a [`PetriNet<Lambda>`] from SurrealDB by its net [`RecordId`].
    ///
    /// Fetches the net header, verifies the stored `label_type` matches
    /// `Lambda::type_name()`, then reconstructs places (ordered by position),
    /// transitions, and their pre/post arcs. Arcs are sorted by place index
    /// after loading to guarantee deterministic [`Transition`] construction.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::NotFound`] if the net record does not exist.
    /// Returns [`PersistError::TypeMismatch`] if `Lambda` does not match the
    /// stored label type. Returns [`PersistError::InvalidData`] on malformed
    /// label JSON or dangling arc references.
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

    /// Save a [`Marking`] snapshot for a Petri net.
    ///
    /// Token counts are serialized as a JSON object mapping place indices
    /// (as string keys) to decimal values (as string values). Zero-count
    /// entries are omitted for compactness. The optional `label` field
    /// allows tagging snapshots (e.g., `"initial"`, `"after_fire_t0"`).
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::InvalidData`] if the marking record cannot be
    /// created. Returns [`PersistError::Surreal`] on database errors.
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

    /// Load a [`Marking`] snapshot by its [`RecordId`].
    ///
    /// Parses the stored JSON token object back into `(place_index, Decimal)`
    /// pairs and constructs a [`Marking`] via [`Marking::from_vec`].
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::NotFound`] if the marking record does not exist.
    /// Returns [`PersistError::InvalidData`] if the token JSON is malformed.
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

    /// Delete a Petri net and all its dependent records.
    ///
    /// Deletes in reverse-dependency order to avoid dangling references:
    /// pre-arcs, post-arcs, transitions, places, markings, then the net
    /// header itself.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::Surreal`] on database communication errors.
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

    /// List all stored Petri net header records.
    ///
    /// Returns [`PetriNetRecord`] values without loading places, transitions,
    /// or arcs -- use [`load`](Self::load) to reconstruct a full
    /// [`PetriNet<Lambda>`].
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::Surreal`] on database communication errors.
    pub async fn list(&self) -> Result<Vec<PetriNetRecord>, PersistError> {
        let records: Vec<PetriNetRecord> = self.db.select("petri_net").await?;
        Ok(records)
    }
}

/// Parse the stored tokens JSON object into `Vec<(usize, Decimal)>` pairs.
///
/// Expects a JSON object of the form `{"0": "3", "2": "1.5"}` where keys
/// are place indices and values are string-encoded [`Decimal`] token counts.
/// Zero-valued entries are silently discarded.
///
/// # Errors
///
/// Returns [`PersistError::InvalidData`] if the value is not a JSON object,
/// a key is not a valid `usize`, or a value is not a parseable decimal string.
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
