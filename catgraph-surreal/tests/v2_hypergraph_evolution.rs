use catgraph::category::Composable;
use catgraph::cospan::Cospan;
use catgraph::hypergraph::{Hypergraph, HypergraphEvolution, RewriteRule};
use catgraph::span::Span;
use catgraph_surreal::hypergraph_evolution_store::HypergraphEvolutionStore;
use catgraph_surreal::init_schema_v2;
use surrealdb::engine::local::{Db, Mem};
use surrealdb::types::RecordId;
use surrealdb::Surreal;

async fn setup() -> Surreal<Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    init_schema_v2(&db).await.unwrap();
    db
}

/// Helper: build a 3-step deterministic evolution using `edge_split` on a simple edge.
fn three_step_evolution() -> HypergraphEvolution {
    let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
    let rules = vec![RewriteRule::edge_split()];
    HypergraphEvolution::run(&initial, &rules, 3)
}

// ---------------------------------------------------------------------------
// Cospan chain roundtrip
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cospan_chain_metadata_roundtrip() {
    let db = setup().await;
    let store = HypergraphEvolutionStore::new(&db);

    let evolution = three_step_evolution();
    let chain = evolution.to_cospan_chain();
    assert_eq!(chain.len(), 3, "edge_split x3 should produce a 3-step chain");

    let hub_ids = store
        .persist_cospan_chain(&evolution, "meta_test")
        .await
        .unwrap();
    assert_eq!(hub_ids.len(), 3);

    // Verify each hub's properties contain correct chain, step, total_steps.
    let he_store = catgraph_surreal::hyperedge_store::HyperedgeStore::new(&db);
    for (i, hub_id) in hub_ids.iter().enumerate() {
        let hub = he_store.get_hub(hub_id).await.unwrap();
        assert_eq!(hub.kind, "evolution_step");
        assert_eq!(hub.properties["chain"], "meta_test");
        assert_eq!(hub.properties["step"], i, "step index mismatch at hub {i}");
        assert_eq!(hub.properties["total_steps"], 3);
    }
}

#[tokio::test]
async fn cospan_chain_reconstruct_preserves_structure() {
    let db = setup().await;
    let store = HypergraphEvolutionStore::new(&db);

    let evolution = three_step_evolution();
    let chain = evolution.to_cospan_chain();
    let hub_ids = store
        .persist_cospan_chain(&evolution, "struct_test")
        .await
        .unwrap();

    for (i, hub_id) in hub_ids.iter().enumerate() {
        let loaded: Cospan<u32> = store.load_cospan(hub_id).await.unwrap();
        let original = &chain[i];

        assert_eq!(
            loaded.middle().len(),
            original.middle().len(),
            "middle size mismatch at step {i}"
        );
        assert_eq!(
            loaded.left_to_middle().len(),
            original.left_to_middle().len(),
            "left_to_middle size mismatch at step {i}"
        );
        assert_eq!(
            loaded.right_to_middle().len(),
            original.right_to_middle().len(),
            "right_to_middle size mismatch at step {i}"
        );
    }
}

#[tokio::test]
async fn cospan_chain_domain_codomain_match() {
    let db = setup().await;
    let store = HypergraphEvolutionStore::new(&db);

    let evolution = three_step_evolution();
    let chain = evolution.to_cospan_chain();

    // Verify composability of the *original* chain (domain property).
    for pair in chain.windows(2) {
        assert_eq!(
            pair[0].codomain(),
            pair[1].domain(),
            "original chain steps are not composable"
        );
    }

    // Now roundtrip and verify the persisted chain preserves the same
    // domain/codomain relationship.
    let hub_ids = store
        .persist_cospan_chain(&evolution, "compose_test")
        .await
        .unwrap();

    let loaded: Vec<Cospan<u32>> = {
        let mut v = Vec::new();
        for hub_id in &hub_ids {
            v.push(store.load_cospan(hub_id).await.unwrap());
        }
        v
    };

    for pair in loaded.windows(2) {
        assert_eq!(
            pair[0].codomain(),
            pair[1].domain(),
            "loaded chain steps are not composable"
        );
    }
}

// ---------------------------------------------------------------------------
// Span roundtrip
// ---------------------------------------------------------------------------

#[tokio::test]
async fn span_roundtrip_preserves_dimensions() {
    let db = setup().await;
    let store = HypergraphEvolutionStore::new(&db);

    let rule = RewriteRule::wolfram_a_to_bb();
    let original: Span<u32> = rule.to_span();
    let hub_id = store.persist_span(&rule, "dim_test").await.unwrap();
    let loaded: Span<u32> = store.load_span(&hub_id).await.unwrap();

    assert_eq!(loaded.left().len(), original.left().len());
    assert_eq!(loaded.right().len(), original.right().len());
    assert_eq!(loaded.middle_pairs().len(), original.middle_pairs().len());
}

#[tokio::test]
async fn span_hub_properties_correct() {
    let db = setup().await;
    let store = HypergraphEvolutionStore::new(&db);
    let he_store = catgraph_surreal::hyperedge_store::HyperedgeStore::new(&db);

    let rule = RewriteRule::wolfram_a_to_bb();
    let span = rule.to_span();
    let hub_id = store.persist_span(&rule, "wolfram_a_to_bb").await.unwrap();

    let hub = he_store.get_hub(&hub_id).await.unwrap();
    assert_eq!(hub.properties["rule_name"], "wolfram_a_to_bb");
    assert_eq!(
        usize::try_from(hub.properties["left_size"].as_u64().unwrap()).unwrap(),
        span.left().len()
    );
    assert_eq!(
        usize::try_from(hub.properties["right_size"].as_u64().unwrap()).unwrap(),
        span.right().len()
    );
    assert_eq!(
        usize::try_from(hub.properties["kernel_size"].as_u64().unwrap()).unwrap(),
        span.middle_pairs().len()
    );
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[tokio::test]
async fn empty_evolution_no_steps() {
    let db = setup().await;
    let store = HypergraphEvolutionStore::new(&db);

    // No rules match: evolution produces zero rewrite steps.
    let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
    let rules = vec![RewriteRule::wolfram_a_to_bb()]; // needs ternary edge, won't match
    let evolution = HypergraphEvolution::run(&initial, &rules, 5);

    let chain = evolution.to_cospan_chain();
    assert!(chain.is_empty(), "no rewrites should produce an empty chain");

    let hub_ids = store
        .persist_cospan_chain(&evolution, "empty_chain")
        .await
        .unwrap();
    assert!(hub_ids.is_empty());
}

#[tokio::test]
async fn single_vertex_hypergraph_evolution() {
    let db = setup().await;
    let store = HypergraphEvolutionStore::new(&db);

    // A single vertex with no edges; no rule can match.
    let initial = Hypergraph::from_edges(Vec::<Vec<usize>>::new());
    let rules = vec![RewriteRule::edge_split()];
    let evolution = HypergraphEvolution::run(&initial, &rules, 3);

    let chain = evolution.to_cospan_chain();
    assert!(chain.is_empty());

    let hub_ids = store
        .persist_cospan_chain(&evolution, "single_vertex")
        .await
        .unwrap();
    assert!(hub_ids.is_empty());
}

// ---------------------------------------------------------------------------
// Isolation: multiple chains coexist independently
// ---------------------------------------------------------------------------

#[tokio::test]
async fn multiple_chains_no_interference() {
    let db = setup().await;
    let store = HypergraphEvolutionStore::new(&db);
    let he_store = catgraph_surreal::hyperedge_store::HyperedgeStore::new(&db);

    // Chain A: edge_split x2
    let evo_a = HypergraphEvolution::run(
        &Hypergraph::from_edges(vec![vec![0, 1]]),
        &[RewriteRule::edge_split()],
        2,
    );
    let ids_a = store
        .persist_cospan_chain(&evo_a, "chain_a")
        .await
        .unwrap();
    assert_eq!(ids_a.len(), 2);

    // Chain B: wolfram_a_to_bb x1
    let evo_b = HypergraphEvolution::run(
        &Hypergraph::from_edges(vec![vec![0, 1, 2]]),
        &[RewriteRule::wolfram_a_to_bb()],
        1,
    );
    let ids_b = store
        .persist_cospan_chain(&evo_b, "chain_b")
        .await
        .unwrap();
    assert_eq!(ids_b.len(), 1);

    // Verify chain_a hubs have chain="chain_a"
    for hub_id in &ids_a {
        let hub = he_store.get_hub(hub_id).await.unwrap();
        assert_eq!(hub.properties["chain"], "chain_a");
    }

    // Verify chain_b hub has chain="chain_b"
    let hub_b = he_store.get_hub(&ids_b[0]).await.unwrap();
    assert_eq!(hub_b.properties["chain"], "chain_b");

    // Loading from chain_a IDs does not return chain_b data
    for hub_id in &ids_a {
        let loaded: Cospan<u32> = store.load_cospan(hub_id).await.unwrap();
        // Chain A's cospan middle sets should not match chain B's.
        // Both are valid cospans; we just verify structural independence.
        let hub = he_store.get_hub(hub_id).await.unwrap();
        assert_ne!(hub.properties["chain"], "chain_b");

        // Check that source/target counts match what we stored
        assert_eq!(
            usize::try_from(hub.source_count).unwrap(),
            loaded.left_to_middle().len()
        );
        assert_eq!(
            usize::try_from(hub.target_count).unwrap(),
            loaded.right_to_middle().len()
        );
    }
}

// ---------------------------------------------------------------------------
// Large-scale evolution
// ---------------------------------------------------------------------------

#[tokio::test]
async fn large_evolution_chain() {
    let db = setup().await;
    let store = HypergraphEvolutionStore::new(&db);

    let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
    let rules = vec![RewriteRule::edge_split()];
    let evolution = HypergraphEvolution::run(&initial, &rules, 12);

    let chain = evolution.to_cospan_chain();
    assert_eq!(chain.len(), 12, "edge_split should apply 12 times");

    let hub_ids = store
        .persist_cospan_chain(&evolution, "large_chain")
        .await
        .unwrap();
    assert_eq!(hub_ids.len(), 12);

    // Verify all hub_ids are valid and step ordering is correct.
    let he_store = catgraph_surreal::hyperedge_store::HyperedgeStore::new(&db);
    for (i, hub_id) in hub_ids.iter().enumerate() {
        let hub = he_store.get_hub(hub_id).await.unwrap();
        assert_eq!(hub.properties["step"], i);
        assert_eq!(hub.properties["total_steps"], 12);
    }

    // Verify the last cospan can be reconstructed.
    let last: Cospan<u32> = store.load_cospan(hub_ids.last().unwrap()).await.unwrap();
    assert!(
        !last.middle().is_empty(),
        "last step should have a non-empty middle set"
    );
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn load_cospan_nonexistent_returns_empty() {
    let db = setup().await;
    let store = HypergraphEvolutionStore::new(&db);

    // reconstruct_cospan queries source_of/target_of edges (not the hub itself),
    // so a nonexistent hub yields an empty cospan rather than an error.
    let fake_id = RecordId::new("hyperedge_hub", "does_not_exist");
    let result = store.load_cospan(&fake_id).await;
    assert!(result.is_ok(), "cospan reconstruct returns Ok for missing hub");
    let cospan = result.unwrap();
    assert!(cospan.middle().is_empty());
    assert!(cospan.left_to_middle().is_empty());
    assert!(cospan.right_to_middle().is_empty());
}

#[tokio::test]
async fn load_span_nonexistent() {
    let db = setup().await;
    let store = HypergraphEvolutionStore::new(&db);

    let fake_id = RecordId::new("hyperedge_hub", "ghost_span");
    let result = store.load_span(&fake_id).await;
    assert!(result.is_err(), "loading a span from a nonexistent hub should fail");
}
