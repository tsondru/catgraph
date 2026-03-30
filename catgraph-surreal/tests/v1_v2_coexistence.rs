//! V1/V2 Coexistence Verification
//!
//! Exercises domain-specific scenarios where both V1 (embedded array) and V2
//! (RELATE-based) persistence layers operate on the same database, verifying
//! mutual non-interference with complex data structures (cospans, spans,
//! named cospans).

use catgraph::cospan::Cospan;
use catgraph::named_cospan::NamedCospan;
use catgraph::span::Span;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

use catgraph_surreal::cospan_store::CospanStore;
use catgraph_surreal::edge_store::EdgeStore;
use catgraph_surreal::hyperedge_store::HyperedgeStore;
use catgraph_surreal::named_cospan_store::NamedCospanStore;
use catgraph_surreal::node_store::NodeStore;
use catgraph_surreal::span_store::SpanStore;
use catgraph_surreal::{init_schema, init_schema_v2};

async fn setup_both() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    init_schema(&db).await.unwrap();
    init_schema_v2(&db).await.unwrap();
    db
}

fn namer<'a>(names: &'a [&'a str]) -> impl Fn(&i32) -> String + 'a {
    move |i: &i32| names[usize::try_from(*i).expect("non-negative index")].to_string()
}

// ---------------------------------------------------------------------------
// 1. Cospan: V1 + V2 independent roundtrip with complex structure
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cospan_complex_both_layers() {
    let db = setup_both().await;
    let v1 = CospanStore::new(&db);
    let v2 = HyperedgeStore::new(&db);

    // Complex cospan: 3 sources, 2 targets, 4 middle nodes, repeated indices
    // left_map: [0, 1, 2] — each source maps to a unique middle node
    // right_map: [2, 3] — targets map to different middle nodes
    let cospan: Cospan<i32> = Cospan::new(vec![0, 1, 2], vec![2, 3], vec![0, 1, 2, 3]);
    let names = ["m0", "m1", "m2", "m3"];

    // V1 save
    let v1_id = v1.save(&cospan).await.unwrap();

    // V2 decompose
    let v2_hub = v2
        .decompose_cospan(&cospan, "complex", serde_json::json!({}), namer(&names))
        .await
        .unwrap();

    // V1 roundtrip
    let v1_loaded: Cospan<i32> = v1.load(&v1_id).await.unwrap();
    assert_eq!(v1_loaded.left_to_middle(), cospan.left_to_middle());
    assert_eq!(v1_loaded.right_to_middle(), cospan.right_to_middle());
    assert_eq!(v1_loaded.middle(), cospan.middle());

    // V2 roundtrip
    let v2_loaded: Cospan<i32> = v2.reconstruct_cospan(&v2_hub).await.unwrap();
    assert_eq!(v2_loaded.left_to_middle(), cospan.left_to_middle());
    assert_eq!(v2_loaded.right_to_middle(), cospan.right_to_middle());
    assert_eq!(v2_loaded.middle(), cospan.middle());
}

// ---------------------------------------------------------------------------
// 2. Span: V1 save + V2 decompose coexist
// ---------------------------------------------------------------------------

#[tokio::test]
async fn span_both_layers() {
    let db = setup_both().await;
    let v1 = SpanStore::new(&db);
    let v2 = HyperedgeStore::new(&db);

    // Span: left=[a, a], right=[a], pairs=[(0,0), (1,0)]
    // Both left elements must match right[0]='a' since both pairs map to right index 0
    let span = Span::new(vec!['a', 'a'], vec!['a'], vec![(0, 0), (1, 0)]);

    // V1 save
    let v1_id = v1.save(&span).await.unwrap();

    // V2 decompose
    let v2_hub = v2
        .decompose_span(&span, "span_test", serde_json::json!({}), char::to_string)
        .await
        .unwrap();

    // V1 roundtrip
    let v1_loaded: Span<char> = v1.load(&v1_id).await.unwrap();
    assert_eq!(v1_loaded.left(), span.left());
    assert_eq!(v1_loaded.right(), span.right());
    assert_eq!(v1_loaded.middle_pairs(), span.middle_pairs());

    // V2 hub has correct counts
    let hub = v2.get_hub(&v2_hub).await.unwrap();
    assert_eq!(hub.source_count, 2); // left side
    assert_eq!(hub.target_count, 1); // right side
}

// ---------------------------------------------------------------------------
// 3. Named cospan: V1 save + V2 decompose preserve port names
// ---------------------------------------------------------------------------

#[tokio::test]
async fn named_cospan_both_layers() {
    let db = setup_both().await;
    let v1 = NamedCospanStore::new(&db);
    let v2 = HyperedgeStore::new(&db);

    let nc: NamedCospan<char, String, String> = NamedCospan::new(
        vec![0, 1],
        vec![2],
        vec!['x', 'y', 'z'],
        vec!["left_port_a".to_string(), "left_port_b".to_string()],
        vec!["right_port".to_string()],
    );

    // V1 save (preserves port names)
    let v1_id = v1.save(&nc).await.unwrap();

    // V2 decompose (underlying cospan only, port names not in V2)
    let v2_hub = v2
        .decompose_named_cospan(&nc, "named_test", serde_json::json!({}))
        .await
        .unwrap();

    // V1 roundtrip with port names
    let v1_loaded: NamedCospan<char, String, String> = v1.load(&v1_id).await.unwrap();
    assert_eq!(v1_loaded.left_names(), nc.left_names());
    assert_eq!(v1_loaded.right_names(), nc.right_names());
    assert_eq!(v1_loaded.cospan().middle(), nc.cospan().middle());

    // V2 roundtrip (cospan structure only)
    let v2_cospan: Cospan<char> = v2.reconstruct_cospan(&v2_hub).await.unwrap();
    assert_eq!(v2_cospan.left_to_middle(), nc.cospan().left_to_middle());
    assert_eq!(v2_cospan.right_to_middle(), nc.cospan().right_to_middle());
}

// ---------------------------------------------------------------------------
// 4. Multiple V1 + V2 records — table isolation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn multiple_records_table_isolation() {
    let db = setup_both().await;
    let v1_cospan = CospanStore::new(&db);
    let v1_span = SpanStore::new(&db);
    let v2 = HyperedgeStore::new(&db);
    let v2_nodes = NodeStore::new(&db);

    // V1: 3 cospans
    for i in 0..3 {
        let c: Cospan<i32> = Cospan::new(vec![0], vec![0], vec![i]);
        v1_cospan.save(&c).await.unwrap();
    }

    // V1: 2 spans
    for _ in 0..2 {
        let s = Span::new(vec!['a'], vec!['a'], vec![(0, 0)]);
        v1_span.save(&s).await.unwrap();
    }

    // V2: 2 hyperedge hubs (each creates middle nodes)
    let c1: Cospan<i32> = Cospan::new(vec![0], vec![1], vec![0, 1]);
    let c2: Cospan<i32> = Cospan::new(vec![0, 1], vec![2], vec![0, 1, 2]);
    v2.decompose_cospan(&c1, "hub1", serde_json::json!({}), i32::to_string)
        .await
        .unwrap();
    v2.decompose_cospan(&c2, "hub2", serde_json::json!({}), i32::to_string)
        .await
        .unwrap();

    // V1 counts are independent of V2
    let v1_ids = v1_cospan.list().await.unwrap();
    assert_eq!(v1_ids.len(), 3, "V1 cospan table should have 3 records");

    let v1_span_ids = v1_span.list().await.unwrap();
    assert_eq!(v1_span_ids.len(), 2, "V1 span table should have 2 records");

    // V2 nodes: c1 creates 2 middle nodes, c2 creates 3 = 5 total
    let v2_node_ids = v2_nodes.list().await.unwrap();
    assert_eq!(v2_node_ids.len(), 5, "V2 should have 5 graph_node records");
}

// ---------------------------------------------------------------------------
// 5. V1 delete doesn't affect V2, and vice versa
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_isolation() {
    let db = setup_both().await;
    let v1 = CospanStore::new(&db);
    let v2 = HyperedgeStore::new(&db);

    let cospan: Cospan<i32> = Cospan::new(vec![0], vec![1], vec![0, 1]);
    let names = ["src", "tgt"];

    let v1_id = v1.save(&cospan).await.unwrap();
    let v2_hub = v2
        .decompose_cospan(&cospan, "delete_test", serde_json::json!({}), namer(&names))
        .await
        .unwrap();

    // Delete V1 record
    v1.delete(&v1_id).await.unwrap();

    // V2 hub still exists
    let hub = v2.get_hub(&v2_hub).await.unwrap();
    assert_eq!(hub.kind, "delete_test");

    // V2 sources/targets still queryable
    let sources = v2.sources(&v2_hub).await.unwrap();
    assert_eq!(sources.len(), 1);

    // Delete V2 hub
    v2.delete_hub(&v2_hub).await.unwrap();

    // V1 list is empty (we deleted the only record)
    let v1_ids = v1.list().await.unwrap();
    assert!(v1_ids.is_empty());
}

// ---------------------------------------------------------------------------
// 6. V2 pairwise edges + V1 cospans in same DB
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pairwise_edges_alongside_v1() {
    let db = setup_both().await;
    let v1 = CospanStore::new(&db);
    let nodes = NodeStore::new(&db);
    let edges = EdgeStore::new(&db);

    // V1: save a cospan
    let cospan: Cospan<char> = Cospan::new(vec![0, 1], vec![2], vec!['a', 'b', 'c']);
    let v1_id = v1.save(&cospan).await.unwrap();

    // V2: create pairwise nodes and edges
    let n1 = nodes
        .create("node_1", "entity", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let n2 = nodes
        .create("node_2", "entity", vec![], serde_json::json!({}))
        .await
        .unwrap();
    edges
        .relate(&n1, &n2, "connects", None, serde_json::json!({}))
        .await
        .unwrap();

    // V1 roundtrip unaffected
    let loaded: Cospan<char> = v1.load(&v1_id).await.unwrap();
    assert_eq!(loaded.middle(), &['a', 'b', 'c']);

    // V2 traversal works
    let neighbors = edges.traverse_outbound(&n1, "connects").await.unwrap();
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0].name, "node_2");
}
