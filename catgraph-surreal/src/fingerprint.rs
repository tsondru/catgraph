use surrealdb::engine::local::Db;
use surrealdb::types::RecordId;
use surrealdb::Surreal;

use crate::error::PersistError;
use crate::types_v2::GraphNodeRecord;

/// Structural fingerprint computation and HNSW similarity search.
///
/// Computes local topology features for graph nodes using SurrealDB queries,
/// stores them as embedding vectors, and searches for structurally similar
/// nodes via HNSW index.
///
/// The embedding dimension is configurable at construction time. Features
/// computed (padded/truncated to dimension):
/// - out-degree, in-degree, total degree
/// - source participation count (hyperedge sources)
/// - target participation count (hyperedge targets)
/// - remaining slots zero-padded
pub struct FingerprintEngine<'a> {
    db: &'a Surreal<Db>,
    dimension: u32,
}

impl<'a> FingerprintEngine<'a> {
    pub fn new(db: &'a Surreal<Db>, dimension: u32) -> Self {
        Self { db, dimension }
    }

    /// Initialize the HNSW index with the configured dimension.
    /// Call this once after init_schema_v2.
    pub async fn init_index(&self) -> Result<(), PersistError> {
        let ddl = crate::schema_v2::hnsw_index_ddl(self.dimension);
        self.db.query(&ddl).await?;
        Ok(())
    }

    /// Compute a structural fingerprint for a node based on its local topology.
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
        let out_degree: f64 = result
            .take::<Option<serde_json::Value>>(0)?
            .and_then(|v| v.get("cnt").and_then(|c| c.as_i64()))
            .unwrap_or(0) as f64;

        // In-degree: edges where this node is the target (`out` field)
        let mut result = self
            .db
            .query("SELECT count() AS cnt FROM graph_edge WHERE out = $node GROUP ALL")
            .bind(("node", node_id.clone()))
            .await?;
        let in_degree: f64 = result
            .take::<Option<serde_json::Value>>(0)?
            .and_then(|v| v.get("cnt").and_then(|c| c.as_i64()))
            .unwrap_or(0) as f64;

        // Hyperedge source participations: source_of edges from this node
        let mut result = self
            .db
            .query("SELECT count() AS cnt FROM source_of WHERE `in` = $node GROUP ALL")
            .bind(("node", node_id.clone()))
            .await?;
        let source_parts: f64 = result
            .take::<Option<serde_json::Value>>(0)?
            .and_then(|v| v.get("cnt").and_then(|c| c.as_i64()))
            .unwrap_or(0) as f64;

        // Hyperedge target participations: target_of edges pointing to this node
        let mut result = self
            .db
            .query("SELECT count() AS cnt FROM target_of WHERE out = $node GROUP ALL")
            .bind(("node", node_id.clone()))
            .await?;
        let target_parts: f64 = result
            .take::<Option<serde_json::Value>>(0)?
            .and_then(|v| v.get("cnt").and_then(|c| c.as_i64()))
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

    /// Store a precomputed fingerprint on a node record.
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

    /// Compute and store fingerprint in one call.
    pub async fn index_node(&self, node_id: &RecordId) -> Result<Vec<f64>, PersistError> {
        let fp = self.compute_fingerprint(node_id).await?;
        self.store_fingerprint(node_id, &fp).await?;
        Ok(fp)
    }

    /// Find the K most structurally similar nodes via HNSW.
    ///
    /// **IMPORTANT**: The query vector must be inlined as a literal in the query
    /// string — bind params in KNN syntax cause a silent table scan fallback.
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
                .map(|x| x.to_string())
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
        let hits: Vec<serde_json::Value> = result.take(0)?;

        let mut results = Vec::with_capacity(hits.len());
        for hit in &hits {
            // Extract fields manually: full deserialization would require
            // `vector::distance::knn()` to be a stable serde field name,
            // and `distance` may be NONE for nodes without an embedding.
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
            let distance = hit
                .get("distance")
                .and_then(|v| v.as_f64())
                .unwrap_or(f64::INFINITY);

            let node = GraphNodeRecord {
                id: None,
                name,
                kind,
                labels: vec![],
                properties: serde_json::json!({}),
                embedding: None,
            };
            results.push((node, distance));
        }
        Ok(results)
    }
}
