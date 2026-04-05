//! SurrealDB persistence for hypergraph evolution traces via V2 hub-node reification.
//!
//! Stores spans, cospans, and full evolution graphs in SurrealDB using
//! [`HyperedgeStore`]'s hub-node decomposition.

use catgraph::cospan::Cospan;
use catgraph::hypergraph::{HypergraphEvolution, RewriteRule};
use catgraph::span::Span;
use surrealdb::engine::local::Db;
use surrealdb::types::RecordId;
use surrealdb::Surreal;

use crate::error::PersistError;
use crate::hyperedge::HyperedgeStore;

/// Persistence layer for hypergraph evolution traces.
///
/// Wraps `HyperedgeStore` to persist spans, cospans, and multiway graphs
/// with evolution-specific metadata in hub properties.
pub struct HypergraphEvolutionStore<'a> {
    store: HyperedgeStore<'a>,
}

impl<'a> HypergraphEvolutionStore<'a> {
    /// Creates a new persistence handle.
    ///
    /// Requires V2 schema to be initialized (`catgraph_surreal::init_schema_v2`).
    pub fn new(db: &'a Surreal<Db>) -> Self {
        Self {
            store: HyperedgeStore::new(db),
        }
    }

    /// Persists the deterministic cospan chain for an evolution.
    ///
    /// Each step cospan is stored as a V2 hub-node decomposition with
    /// properties: `chain_name`, `step`, `total_steps`.
    pub async fn persist_cospan_chain(
        &self,
        evolution: &HypergraphEvolution,
        chain_name: &str,
    ) -> Result<Vec<RecordId>, PersistError> {
        let chain = evolution.to_cospan_chain();
        let total = chain.len();
        let mut hub_ids = Vec::with_capacity(total);

        for (i, cospan) in chain.iter().enumerate() {
            let props = serde_json::json!({
                "chain": chain_name,
                "step": i,
                "total_steps": total,
            });
            let hub_id = self.store.decompose_cospan(
                cospan,
                "evolution_step",
                props,
                |v: &u32| format!("v{v}"),
            ).await?;
            hub_ids.push(hub_id);
        }

        Ok(hub_ids)
    }

    /// Persists a rewrite rule as a categorical span.
    pub async fn persist_span(
        &self,
        rule: &RewriteRule,
        rule_name: &str,
    ) -> Result<RecordId, PersistError> {
        let span = rule.to_span();
        let props = serde_json::json!({
            "rule_name": rule_name,
            "left_size": span.left().len(),
            "right_size": span.right().len(),
            "kernel_size": span.middle_pairs().len(),
        });
        self.store.decompose_span(
            &span,
            "rewrite_rule",
            props,
            |v: &u32| format!("v{v}"),
        ).await
    }

    /// Reconstructs a cospan from a persisted hub record.
    pub async fn load_cospan(
        &self,
        hub_id: &RecordId,
    ) -> Result<Cospan<u32>, PersistError> {
        self.store.reconstruct_cospan::<u32>(hub_id).await
    }

    /// Reconstructs a rewrite rule span from a persisted hub record.
    pub async fn load_span(
        &self,
        hub_id: &RecordId,
    ) -> Result<Span<u32>, PersistError> {
        self.store.reconstruct_span::<u32>(hub_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use catgraph::hypergraph::{Hypergraph, HypergraphEvolution, RewriteRule};
    use surrealdb::engine::local::Mem;

    async fn setup_db() -> Surreal<Db> {
        let db = surrealdb::Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        crate::init_schema_v2(&db).await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_persist_single_step() {
        let db = setup_db().await;
        let persist = HypergraphEvolutionStore::new(&db);

        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 1);

        let hub_ids = persist.persist_cospan_chain(&evolution, "test_chain").await.unwrap();
        assert_eq!(hub_ids.len(), 1);
    }

    #[tokio::test]
    async fn test_persist_multi_step() {
        let db = setup_db().await;
        let persist = HypergraphEvolutionStore::new(&db);

        let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
        let rules = vec![RewriteRule::edge_split()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 3);

        let hub_ids = persist.persist_cospan_chain(&evolution, "split_chain").await.unwrap();
        assert_eq!(hub_ids.len(), 3);
    }

    #[tokio::test]
    async fn test_persist_and_reconstruct() {
        let db = setup_db().await;
        let persist = HypergraphEvolutionStore::new(&db);

        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 1);

        let chain = evolution.to_cospan_chain();
        let hub_ids = persist.persist_cospan_chain(&evolution, "roundtrip").await.unwrap();

        let loaded = persist.load_cospan(&hub_ids[0]).await.unwrap();
        assert_eq!(loaded.middle().len(), chain[0].middle().len());
        assert_eq!(loaded.left_to_middle().len(), chain[0].left_to_middle().len());
    }

    #[tokio::test]
    async fn test_persist_span() {
        let db = setup_db().await;
        let persist = HypergraphEvolutionStore::new(&db);

        let rule = RewriteRule::wolfram_a_to_bb();
        let hub_id = persist.persist_span(&rule, "a_to_bb").await.unwrap();

        let hub = persist.store.get_hub(&hub_id).await.unwrap();
        // A→BB has 3 left (source) and 3 right (target) nodes
        assert_eq!(hub.source_count, 3);
        assert_eq!(hub.target_count, 3);
    }

    #[tokio::test]
    async fn test_persist_and_load_span() {
        let db = setup_db().await;
        let persist = HypergraphEvolutionStore::new(&db);

        let rule = RewriteRule::wolfram_a_to_bb();
        let original_span = rule.to_span();
        let hub_id = persist.persist_span(&rule, "roundtrip_span").await.unwrap();

        let loaded = persist.load_span(&hub_id).await.unwrap();
        // Compare structural properties (Span may not implement PartialEq)
        assert_eq!(loaded.left().len(), original_span.left().len());
        assert_eq!(loaded.right().len(), original_span.right().len());
        assert_eq!(loaded.middle_pairs().len(), original_span.middle_pairs().len());
    }

    #[tokio::test]
    async fn test_load_span_nonexistent() {
        let db = setup_db().await;
        let persist = HypergraphEvolutionStore::new(&db);

        let fake_id = RecordId::new("hub", "nonexistent");
        let result = persist.load_span(&fake_id).await;
        assert!(result.is_err());
    }
}
