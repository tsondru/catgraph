//! Structural fingerprint computation and HNSW similarity search.
//!
//! Computes local topology features for V2 graph nodes and stores them as
//! embedding vectors on `graph_node` records. An HNSW index enables efficient
//! K-nearest-neighbor queries over those embeddings, surfacing nodes with
//! similar structural roles (same degree profile, same hyperedge participation
//! pattern) regardless of label or name.
//!
//! Feature vector layout (padded/truncated to the configured dimension):
//! `[out_degree, in_degree, total_degree, source_participations, target_participations, 0, ...]`

use surrealdb::engine::local::Db;
use surrealdb::types::RecordId;
use surrealdb::Surreal;
use surrealdb_types::SurrealValue;

use crate::error::PersistError;
use crate::types_v2::GraphNodeRecord;

/// Private deserialization target for KNN query results.
///
/// Mirrors all [`GraphNodeRecord`] fields plus the computed `distance`
/// alias returned by `vector::distance::knn() AS distance`.  Every field
/// carries `#[serde(default)]` so that server-managed fields absent from a
/// particular result row do not cause deserialization failures.
///
/// `SurrealValue` is derived so that `result.take::<Vec<SimilarNodeHit>>`
/// works — this routes `RecordId` through the SDK's own deserialization path
/// rather than through `serde_json::from_value`, which cannot round-trip
/// `SurrealDB`'s internal `RecordId` JSON encoding.
#[derive(Debug, serde::Deserialize, SurrealValue)]
struct SimilarNodeHit {
    #[serde(default)]
    id: Option<RecordId>,
    #[serde(default)]
    name: String,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    properties: serde_json::Value,
    #[serde(default)]
    embedding: Option<Vec<f64>>,
    #[serde(default)]
    distance: f64,
}

impl From<SimilarNodeHit> for (GraphNodeRecord, f64) {
    fn from(h: SimilarNodeHit) -> Self {
        let node = GraphNodeRecord {
            id: h.id,
            name: h.name,
            kind: h.kind,
            labels: h.labels,
            properties: h.properties,
            embedding: h.embedding,
        };
        (node, h.distance)
    }
}

/// Engine for computing structural fingerprints and running HNSW similarity
/// searches over V2 graph nodes.
///
/// Each node's fingerprint is a fixed-length `f64` vector capturing local
/// topology: degree counts (out, in, total) and hyperedge participation
/// counts (source, target). The vector is zero-padded or truncated to the
/// configured `dimension` and stored as an `embedding` field on the
/// `graph_node` record. An HNSW index (initialized via [`init_index`](Self::init_index))
/// enables sub-linear K-nearest-neighbor queries over those embeddings.
pub struct FingerprintEngine<'a> {
    /// Borrowed database connection used for all queries.
    db: &'a Surreal<Db>,
    /// Fixed dimension of the embedding vectors and HNSW index.
    dimension: u32,
}

impl<'a> FingerprintEngine<'a> {
    #[must_use] 
    pub fn new(db: &'a Surreal<Db>, dimension: u32) -> Self {
        Self { db, dimension }
    }

    /// Create the HNSW index on `graph_node.embedding` with the configured
    /// dimension.
    ///
    /// Must be called once after [`init_schema_v2`](crate::init_schema_v2).
    /// Subsequent calls are idempotent (`SurrealDB`'s `DEFINE INDEX ... IF NOT
    /// EXISTS` semantics).
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::Surreal`] if the DDL execution fails.
    pub async fn init_index(&self) -> Result<(), PersistError> {
        let ddl = crate::schema_v2::hnsw_index_ddl(self.dimension);
        self.db.query(&ddl).await?;
        Ok(())
    }

    /// Compute a structural fingerprint for a single node.
    ///
    /// Issues four independent `SELECT count()` queries to gather:
    /// 1. **Out-degree** -- `graph_edge` rows where this node is `in` (source).
    /// 2. **In-degree** -- `graph_edge` rows where this node is `out` (target).
    /// 3. **Source participations** -- `source_of` edges from this node to
    ///    hyperedge hubs.
    /// 4. **Target participations** -- `target_of` edges pointing to this node
    ///    from hyperedge hubs.
    ///
    /// The fifth feature is total degree (out + in). The resulting vector is
    /// zero-padded to [`dimension`](Self::new).
    ///
    /// Does **not** store the fingerprint -- call [`store_fingerprint`](Self::store_fingerprint)
    /// or use the convenience method [`index_node`](Self::index_node).
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::Surreal`] on database communication errors.
    pub async fn compute_fingerprint(
        &self,
        node_id: &RecordId,
    ) -> Result<Vec<f64>, PersistError> {
        // Out-degree: edges where this node is the source (`in` field)
        let mut result = self
            .db
            .query("SELECT count() AS cnt FROM graph_edge WHERE `in` = $node GROUP ALL")
            .bind(("node", node_id.clone()))
            .await?;
        // Degree counts from SurrealDB are small integers that fit in f64 without
        // precision loss. The i64→f64 cast is safe for values below 2^53.
        #[allow(clippy::cast_precision_loss)]
        let out_degree: f64 = result
            .take::<Option<serde_json::Value>>(0)?
            .and_then(|v| v.get("cnt").and_then(serde_json::Value::as_i64))
            .unwrap_or(0) as f64;

        // In-degree: edges where this node is the target (`out` field)
        let mut result = self
            .db
            .query("SELECT count() AS cnt FROM graph_edge WHERE out = $node GROUP ALL")
            .bind(("node", node_id.clone()))
            .await?;
        #[allow(clippy::cast_precision_loss)]
        let in_degree: f64 = result
            .take::<Option<serde_json::Value>>(0)?
            .and_then(|v| v.get("cnt").and_then(serde_json::Value::as_i64))
            .unwrap_or(0) as f64;

        // Hyperedge source participations: source_of edges from this node
        let mut result = self
            .db
            .query("SELECT count() AS cnt FROM source_of WHERE `in` = $node GROUP ALL")
            .bind(("node", node_id.clone()))
            .await?;
        #[allow(clippy::cast_precision_loss)]
        let source_parts: f64 = result
            .take::<Option<serde_json::Value>>(0)?
            .and_then(|v| v.get("cnt").and_then(serde_json::Value::as_i64))
            .unwrap_or(0) as f64;

        // Hyperedge target participations: target_of edges pointing to this node
        let mut result = self
            .db
            .query("SELECT count() AS cnt FROM target_of WHERE out = $node GROUP ALL")
            .bind(("node", node_id.clone()))
            .await?;
        #[allow(clippy::cast_precision_loss)]
        let target_parts: f64 = result
            .take::<Option<serde_json::Value>>(0)?
            .and_then(|v| v.get("cnt").and_then(serde_json::Value::as_i64))
            .unwrap_or(0) as f64;

        let mut features = vec![
            out_degree,
            in_degree,
            out_degree + in_degree, // total degree
            source_parts,
            target_parts,
        ];
        features.resize(self.dimension as usize, 0.0);
        Ok(features)
    }

    /// Persist a precomputed fingerprint on a node's `embedding` field.
    ///
    /// Overwrites any existing embedding. The vector length should match
    /// the HNSW index dimension; mismatches may cause index lookup failures.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::Surreal`] on database communication errors.
    pub async fn store_fingerprint(
        &self,
        node_id: &RecordId,
        fingerprint: &[f64],
    ) -> Result<(), PersistError> {
        self.db
            .query("UPDATE $node SET embedding = $emb")
            .bind(("node", node_id.clone()))
            .bind(("emb", fingerprint.to_vec()))
            .await?;
        Ok(())
    }

    /// Compute a fingerprint and immediately store it on the node.
    ///
    /// Convenience wrapper combining [`compute_fingerprint`](Self::compute_fingerprint)
    /// and [`store_fingerprint`](Self::store_fingerprint). Returns the
    /// computed vector for caller inspection.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::Surreal`] on database communication errors.
    pub async fn index_node(&self, node_id: &RecordId) -> Result<Vec<f64>, PersistError> {
        let fp = self.compute_fingerprint(node_id).await?;
        self.store_fingerprint(node_id, &fp).await?;
        Ok(fp)
    }

    /// Find the `k` most structurally similar nodes via HNSW nearest-neighbor
    /// search.
    ///
    /// Returns `(GraphNodeRecord, distance)` pairs ordered by ascending
    /// distance (closest first). The `ef` parameter controls the HNSW search
    /// beam width -- higher values improve recall at the cost of latency.
    ///
    /// **Implementation note**: The query vector is inlined as a literal in
    /// the `SurrealQL` string rather than passed as a bind parameter, because
    /// `SurrealDB`'s KNN operator `<|k,ef|>` silently falls back to a full
    /// table scan when the vector is a bind variable.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::Surreal`] on database communication errors.
    pub async fn search_similar(
        &self,
        query_vector: &[f64],
        k: usize,
        ef: usize,
    ) -> Result<Vec<(GraphNodeRecord, f64)>, PersistError> {
        let vec_literal: String = format!(
            "[{}]",
            query_vector
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
        let query = format!(
            "SELECT *, vector::distance::knn() AS distance \
             FROM graph_node \
             WHERE embedding <|{k},{ef}|> {vec_literal} \
             ORDER BY distance ASC"
        );
        let mut result = self.db.query(&query).await?;

        // Use `result.take::<Vec<SimilarNodeHit>>` so that `RecordId` is
        // decoded by the SDK's own deserialization path (via `SurrealValue`).
        // Going through `serde_json::Value` as an intermediate breaks `RecordId`
        // because SurrealDB's internal JSON encoding for record IDs is not a
        // plain JSON value that serde_json can round-trip.
        //
        // `SimilarNodeHit` derives `SurrealValue` and carries `#[serde(default)]`
        // on every field, so the extra `distance` alias and any other
        // server-managed fields are handled gracefully.  If the typed take
        // fails (e.g. a future schema incompatibility), we re-run the query
        // and fall back to manual JSON extraction for all fields.
        let results: Vec<(GraphNodeRecord, f64)> =
            if let Ok(hits) = result.take::<Vec<SimilarNodeHit>>(0) {
                hits.into_iter().map(Into::into).collect()
            } else {
                // Fallback: re-run and extract all fields from raw JSON.
                // `id` cannot be recovered via serde_json (RecordId encoding is
                // SDK-internal), so it remains None in this path.
                let mut r2 = self.db.query(&query).await?;
                let raw: Vec<serde_json::Value> = r2.take(0)?;
                raw.into_iter()
                    .map(|hit| {
                        let name = hit
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let kind = hit
                            .get("kind")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let labels: Vec<String> = hit
                            .get("labels")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();
                        let properties = hit
                            .get("properties")
                            .cloned()
                            .unwrap_or(serde_json::json!({}));
                        let embedding: Option<Vec<f64>> = hit
                            .get("embedding")
                            .and_then(|v| serde_json::from_value(v.clone()).ok());
                        let distance = hit
                            .get("distance")
                            .and_then(serde_json::Value::as_f64)
                            .unwrap_or(f64::INFINITY);
                        let node = GraphNodeRecord {
                            id: None,
                            name,
                            kind,
                            labels,
                            properties,
                            embedding,
                        };
                        (node, distance)
                    })
                    .collect()
            };
        Ok(results)
    }
}
