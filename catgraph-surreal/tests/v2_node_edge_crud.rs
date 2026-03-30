use catgraph::cospan::Cospan;
use catgraph::span::Span;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

use catgraph_surreal::cospan_store::CospanStore;
use catgraph_surreal::edge_store::EdgeStore;
use catgraph_surreal::hyperedge_store::HyperedgeStore;
use catgraph_surreal::node_store::NodeStore;
use catgraph_surreal::query::QueryHelper;
use catgraph_surreal::{init_schema, init_schema_v2};

async fn setup_v2() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    init_schema_v2(&db).await.unwrap();
    db
}

async fn setup_both() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    init_schema(&db).await.unwrap();
    init_schema_v2(&db).await.unwrap();
    db
}

// ---------------------------------------------------------------------------
// 1. Node CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn node_create_and_verify_fields() {
    let db = setup_v2().await;
    let store = NodeStore::new(&db);

    let id = store
        .create(
            "alpha",
            "process",
            vec!["tag1".into()],
            serde_json::json!({"weight": 1.5}),
        )
        .await
        .unwrap();

    let node = store.get(&id).await.unwrap();
    assert_eq!(node.name, "alpha");
    assert_eq!(node.kind, "process");
    assert_eq!(node.labels, vec!["tag1".to_string()]);
    assert_eq!(node.properties["weight"], 1.5);
}

#[tokio::test]
async fn node_get_by_id() {
    let db = setup_v2().await;
    let store = NodeStore::new(&db);

    let id = store
        .create("beta", "entity", vec![], serde_json::json!({}))
        .await
        .unwrap();

    let node = store.get(&id).await.unwrap();
    assert_eq!(node.name, "beta");
    assert_eq!(node.id.as_ref().unwrap(), &id);
}

#[tokio::test]
async fn node_update_fields() {
    let db = setup_v2().await;
    let store = NodeStore::new(&db);

    let id = store
        .create("gamma", "old_kind", vec![], serde_json::json!({}))
        .await
        .unwrap();

    let updated = store
        .update(
            &id,
            "gamma_renamed",
            "new_kind",
            vec!["updated".into()],
            serde_json::json!({"version": 2}),
        )
        .await
        .unwrap();

    assert_eq!(updated.name, "gamma_renamed");
    assert_eq!(updated.kind, "new_kind");
    assert_eq!(updated.labels, vec!["updated".to_string()]);
    assert_eq!(updated.properties["version"], 2);

    // Re-fetch to confirm persistence
    let refetched = store.get(&id).await.unwrap();
    assert_eq!(refetched.name, "gamma_renamed");
}

#[tokio::test]
async fn node_delete_and_verify_gone() {
    let db = setup_v2().await;
    let store = NodeStore::new(&db);

    let id = store
        .create("ephemeral", "temp", vec![], serde_json::json!({}))
        .await
        .unwrap();

    store.delete(&id).await.unwrap();

    let result = store.get(&id).await;
    assert!(result.is_err(), "deleted node should not be found");
}

#[tokio::test]
async fn node_find_by_kind() {
    let db = setup_v2().await;
    let store = NodeStore::new(&db);

    store
        .create("a", "service", vec![], serde_json::json!({}))
        .await
        .unwrap();
    store
        .create("b", "service", vec![], serde_json::json!({}))
        .await
        .unwrap();
    store
        .create("c", "database", vec![], serde_json::json!({}))
        .await
        .unwrap();

    let services = store.find_by_kind("service").await.unwrap();
    assert_eq!(services.len(), 2);

    let databases = store.find_by_kind("database").await.unwrap();
    assert_eq!(databases.len(), 1);
}

#[tokio::test]
async fn node_find_by_name() {
    let db = setup_v2().await;
    let store = NodeStore::new(&db);

    store
        .create("unique_name", "kind", vec![], serde_json::json!({}))
        .await
        .unwrap();
    store
        .create("other", "kind", vec![], serde_json::json!({}))
        .await
        .unwrap();

    let found = store.find_by_name("unique_name").await.unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "unique_name");

    let not_found = store.find_by_name("nonexistent").await.unwrap();
    assert!(not_found.is_empty());
}

// ---------------------------------------------------------------------------
// 2. Edge RELATE + traversal
// ---------------------------------------------------------------------------

#[tokio::test]
async fn edge_relate_and_verify_fields() {
    let db = setup_v2().await;
    let nodes = NodeStore::new(&db);
    let edges = EdgeStore::new(&db);

    let a = nodes.create("A", "fn", vec![], serde_json::json!({})).await.unwrap();
    let b = nodes.create("B", "fn", vec![], serde_json::json!({})).await.unwrap();
    let c = nodes.create("C", "fn", vec![], serde_json::json!({})).await.unwrap();

    let ab_id = edges
        .relate(&a, &b, "calls", Some(1.0), serde_json::json!({"async": true}))
        .await
        .unwrap();
    let bc_id = edges
        .relate(&b, &c, "calls", Some(2.5), serde_json::json!({}))
        .await
        .unwrap();

    // Verify edge fields
    let ab = edges.get(&ab_id).await.unwrap();
    assert_eq!(ab.kind, "calls");
    assert_eq!(ab.weight, Some(1.0));
    assert_eq!(ab.properties["async"], true);

    let bc = edges.get(&bc_id).await.unwrap();
    assert_eq!(bc.kind, "calls");
    assert_eq!(bc.weight, Some(2.5));
}

#[tokio::test]
async fn edge_traverse_outbound() {
    let db = setup_v2().await;
    let nodes = NodeStore::new(&db);
    let edges = EdgeStore::new(&db);

    let a = nodes.create("A", "fn", vec![], serde_json::json!({})).await.unwrap();
    let b = nodes.create("B", "fn", vec![], serde_json::json!({})).await.unwrap();
    let c = nodes.create("C", "fn", vec![], serde_json::json!({})).await.unwrap();

    edges.relate(&a, &b, "calls", None, serde_json::json!({})).await.unwrap();
    edges.relate(&b, &c, "calls", None, serde_json::json!({})).await.unwrap();

    let from_a = edges.traverse_outbound(&a, "calls").await.unwrap();
    assert_eq!(from_a.len(), 1);
    assert_eq!(from_a[0].name, "B");
}

#[tokio::test]
async fn edge_traverse_inbound() {
    let db = setup_v2().await;
    let nodes = NodeStore::new(&db);
    let edges = EdgeStore::new(&db);

    let a = nodes.create("A", "fn", vec![], serde_json::json!({})).await.unwrap();
    let b = nodes.create("B", "fn", vec![], serde_json::json!({})).await.unwrap();
    let c = nodes.create("C", "fn", vec![], serde_json::json!({})).await.unwrap();

    edges.relate(&a, &b, "calls", None, serde_json::json!({})).await.unwrap();
    edges.relate(&b, &c, "calls", None, serde_json::json!({})).await.unwrap();

    let to_c = edges.traverse_inbound(&c, "calls").await.unwrap();
    assert_eq!(to_c.len(), 1);
    assert_eq!(to_c[0].name, "B");
}

#[tokio::test]
async fn edge_edges_between() {
    let db = setup_v2().await;
    let nodes = NodeStore::new(&db);
    let edges = EdgeStore::new(&db);

    let a = nodes.create("A", "fn", vec![], serde_json::json!({})).await.unwrap();
    let b = nodes.create("B", "fn", vec![], serde_json::json!({})).await.unwrap();

    edges.relate(&a, &b, "calls", None, serde_json::json!({})).await.unwrap();

    let between = edges.edges_between(&a, &b).await.unwrap();
    assert_eq!(between.len(), 1);
    assert_eq!(between[0].kind, "calls");

    // No edges in the reverse direction
    let reverse = edges.edges_between(&b, &a).await.unwrap();
    assert!(reverse.is_empty());
}

// ---------------------------------------------------------------------------
// 3. Hyperedge decompose + reconstruct (Cospan roundtrip)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn hyperedge_cospan_decompose_and_sources_targets() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    // Cospan: {a,b} -> {x,y,z} <- {c}
    // left maps: a->x(0), b->y(1)
    // right maps: c->z(2)
    let cospan = Cospan::new(vec![0, 1], vec![2], vec!['x', 'y', 'z']);

    let hub_id = store
        .decompose_cospan(&cospan, "test_cospan", serde_json::json!({}), |c| {
            c.to_string()
        })
        .await
        .unwrap();

    // Verify sources (left side): 2 entries
    let sources = store.sources(&hub_id).await.unwrap();
    assert_eq!(sources.len(), 2, "expected 2 source nodes");

    // Verify targets (right side): 1 entry
    let targets = store.targets(&hub_id).await.unwrap();
    assert_eq!(targets.len(), 1, "expected 1 target node");

    // Verify hub metadata
    let hub = store.get_hub(&hub_id).await.unwrap();
    assert_eq!(hub.kind, "test_cospan");
    assert_eq!(hub.source_count, 2);
    assert_eq!(hub.target_count, 1);
}

#[tokio::test]
async fn hyperedge_cospan_reconstruct() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    let cospan = Cospan::new(vec![0, 1], vec![2], vec!['x', 'y', 'z']);

    let hub_id = store
        .decompose_cospan(&cospan, "roundtrip", serde_json::json!({}), |c| {
            c.to_string()
        })
        .await
        .unwrap();

    let reconstructed: Cospan<char> = store.reconstruct_cospan(&hub_id).await.unwrap();

    assert_eq!(reconstructed.left_to_middle(), cospan.left_to_middle());
    assert_eq!(reconstructed.right_to_middle(), cospan.right_to_middle());
    assert_eq!(reconstructed.middle(), cospan.middle());
}

#[tokio::test]
async fn hyperedge_cospan_single_middle_node() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    // Both sides map to the same single middle node
    let cospan = Cospan::new(vec![0, 0], vec![0], vec!['m']);

    let hub_id = store
        .decompose_cospan(&cospan, "single", serde_json::json!({}), |c| c.to_string())
        .await
        .unwrap();

    let reconstructed: Cospan<char> = store.reconstruct_cospan(&hub_id).await.unwrap();

    assert_eq!(reconstructed.left_to_middle(), &[0, 0]);
    assert_eq!(reconstructed.right_to_middle(), &[0]);
    assert_eq!(reconstructed.middle(), &['m']);
}

// ---------------------------------------------------------------------------
// 4. Span decomposition
// ---------------------------------------------------------------------------

#[tokio::test]
async fn hyperedge_span_decompose() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    // Span: left=[a, b], right=[a], middle_pairs=[(0,0), (1,0)]
    // Note: Span requires left[i] == right[j] for each pair (i,j), so right must be 'a'
    // since both middle pairs map to right[0] and left[0]='a', left[1]='b' -- but 'b' != 'a'.
    // Use matching labels: left=[a, a], right=[a], pairs=[(0,0),(1,0)]
    let span = Span::new(vec!['a', 'a'], vec!['a'], vec![(0, 0), (1, 0)]);

    let hub_id = store
        .decompose_span(&span, "test_span", serde_json::json!({}), |c| c.to_string())
        .await
        .unwrap();

    // Sources = left side = 2 nodes
    let sources = store.sources(&hub_id).await.unwrap();
    assert_eq!(sources.len(), 2, "expected 2 source nodes from span left");

    // Targets = right side = 1 node
    let targets = store.targets(&hub_id).await.unwrap();
    assert_eq!(targets.len(), 1, "expected 1 target node from span right");

    // Verify hub metadata
    let hub = store.get_hub(&hub_id).await.unwrap();
    assert_eq!(hub.kind, "test_span");
    assert_eq!(hub.source_count, 2);
    assert_eq!(hub.target_count, 1);
}

#[tokio::test]
async fn hyperedge_span_labels_preserved() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    // Span requires left[i] == right[j] for each pair (i,j)
    let span = Span::new(vec!['a', 'a'], vec!['a'], vec![(0, 0), (1, 0)]);

    let hub_id = store
        .decompose_span(&span, "labelled", serde_json::json!({}), |c| c.to_string())
        .await
        .unwrap();

    let sources = store.sources(&hub_id).await.unwrap();
    let source_names: Vec<&str> = sources.iter().map(|n| n.name.as_str()).collect();
    assert_eq!(source_names, vec!["a", "a"]);

    let targets = store.targets(&hub_id).await.unwrap();
    assert_eq!(targets[0].name, "a");
}

#[tokio::test]
async fn hyperedge_hub_delete() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    let cospan = Cospan::new(vec![0], vec![0], vec!['z']);
    let hub_id = store
        .decompose_cospan(&cospan, "deleteme", serde_json::json!({}), |c| c.to_string())
        .await
        .unwrap();

    // Hub exists
    store.get_hub(&hub_id).await.unwrap();

    // Delete hub and its participation edges
    store.delete_hub(&hub_id).await.unwrap();

    let result = store.get_hub(&hub_id).await;
    assert!(result.is_err(), "deleted hub should not be found");
}

// ---------------------------------------------------------------------------
// 5. QueryHelper
// ---------------------------------------------------------------------------

#[tokio::test]
async fn query_outbound_neighbors() {
    let db = setup_v2().await;
    let nodes = NodeStore::new(&db);
    let edges = EdgeStore::new(&db);
    let query = QueryHelper::new(&db);

    let a = nodes.create("A", "fn", vec![], serde_json::json!({})).await.unwrap();
    let b = nodes.create("B", "fn", vec![], serde_json::json!({})).await.unwrap();
    let c = nodes.create("C", "fn", vec![], serde_json::json!({})).await.unwrap();

    edges.relate(&a, &b, "calls", None, serde_json::json!({})).await.unwrap();
    edges.relate(&b, &c, "calls", None, serde_json::json!({})).await.unwrap();

    let neighbors = query.outbound_neighbors(&a, "calls").await.unwrap();
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0].name, "B");
}

#[tokio::test]
async fn query_inbound_neighbors() {
    let db = setup_v2().await;
    let nodes = NodeStore::new(&db);
    let edges = EdgeStore::new(&db);
    let query = QueryHelper::new(&db);

    let a = nodes.create("A", "fn", vec![], serde_json::json!({})).await.unwrap();
    let b = nodes.create("B", "fn", vec![], serde_json::json!({})).await.unwrap();
    let c = nodes.create("C", "fn", vec![], serde_json::json!({})).await.unwrap();

    edges.relate(&a, &b, "calls", None, serde_json::json!({})).await.unwrap();
    edges.relate(&b, &c, "calls", None, serde_json::json!({})).await.unwrap();

    let neighbors = query.inbound_neighbors(&c, "calls").await.unwrap();
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0].name, "B");
}

#[tokio::test]
async fn query_reachable_depth_2() {
    let db = setup_v2().await;
    let nodes = NodeStore::new(&db);
    let edges = EdgeStore::new(&db);
    let query = QueryHelper::new(&db);

    let a = nodes.create("A", "fn", vec![], serde_json::json!({})).await.unwrap();
    let b = nodes.create("B", "fn", vec![], serde_json::json!({})).await.unwrap();
    let c = nodes.create("C", "fn", vec![], serde_json::json!({})).await.unwrap();

    edges.relate(&a, &b, "calls", None, serde_json::json!({})).await.unwrap();
    edges.relate(&b, &c, "calls", None, serde_json::json!({})).await.unwrap();

    let reachable = query.reachable(&a, "calls", 2).await.unwrap();
    assert_eq!(reachable.len(), 2);
    let names: Vec<&str> = reachable.iter().map(|n| n.name.as_str()).collect();
    assert!(names.contains(&"B"));
    assert!(names.contains(&"C"));
}

#[tokio::test]
async fn query_reachable_depth_1_limited() {
    let db = setup_v2().await;
    let nodes = NodeStore::new(&db);
    let edges = EdgeStore::new(&db);
    let query = QueryHelper::new(&db);

    let a = nodes.create("A", "fn", vec![], serde_json::json!({})).await.unwrap();
    let b = nodes.create("B", "fn", vec![], serde_json::json!({})).await.unwrap();
    let c = nodes.create("C", "fn", vec![], serde_json::json!({})).await.unwrap();

    edges.relate(&a, &b, "calls", None, serde_json::json!({})).await.unwrap();
    edges.relate(&b, &c, "calls", None, serde_json::json!({})).await.unwrap();

    // Depth 1: only B reachable from A
    let reachable = query.reachable(&a, "calls", 1).await.unwrap();
    assert_eq!(reachable.len(), 1);
    assert_eq!(reachable[0].name, "B");
}

#[tokio::test]
async fn query_reachable_wrong_kind_empty() {
    let db = setup_v2().await;
    let nodes = NodeStore::new(&db);
    let edges = EdgeStore::new(&db);
    let query = QueryHelper::new(&db);

    let a = nodes.create("A", "fn", vec![], serde_json::json!({})).await.unwrap();
    let b = nodes.create("B", "fn", vec![], serde_json::json!({})).await.unwrap();

    edges.relate(&a, &b, "calls", None, serde_json::json!({})).await.unwrap();

    // Wrong edge kind yields empty result
    let reachable = query.reachable(&a, "imports", 5).await.unwrap();
    assert!(reachable.is_empty());
}

// ---------------------------------------------------------------------------
// 6. V1/V2 coexistence
// ---------------------------------------------------------------------------

#[tokio::test]
async fn v1_v2_coexistence_both_schemas() {
    let db = setup_both().await;

    // V1: save a cospan via CospanStore (embedded arrays)
    let v1_store = CospanStore::new(&db);
    let cospan = Cospan::new(vec![0, 1], vec![2], vec!['x', 'y', 'z']);
    let v1_id = v1_store.save::<char>(&cospan).await.unwrap();

    // V2: decompose the same cospan via HyperedgeStore (RELATE-based)
    let v2_store = HyperedgeStore::new(&db);
    let v2_hub = v2_store
        .decompose_cospan(&cospan, "coexist", serde_json::json!({}), |c| c.to_string())
        .await
        .unwrap();

    // V1 load still works
    let v1_loaded: Cospan<char> = v1_store.load(&v1_id).await.unwrap();
    assert_eq!(v1_loaded.left_to_middle(), cospan.left_to_middle());
    assert_eq!(v1_loaded.right_to_middle(), cospan.right_to_middle());
    assert_eq!(v1_loaded.middle(), cospan.middle());

    // V2 reconstruct still works
    let v2_reconstructed: Cospan<char> = v2_store.reconstruct_cospan(&v2_hub).await.unwrap();
    assert_eq!(v2_reconstructed.left_to_middle(), cospan.left_to_middle());
    assert_eq!(v2_reconstructed.right_to_middle(), cospan.right_to_middle());
    assert_eq!(v2_reconstructed.middle(), cospan.middle());
}

#[tokio::test]
async fn v1_v2_no_table_interference() {
    let db = setup_both().await;

    // V1: create two cospans
    let v1_store = CospanStore::new(&db);
    v1_store
        .save::<char>(&Cospan::new(vec![0], vec![0], vec!['a']))
        .await
        .unwrap();
    v1_store
        .save::<char>(&Cospan::new(vec![0], vec![0], vec!['b']))
        .await
        .unwrap();

    // V2: create one hyperedge hub
    let v2_store = HyperedgeStore::new(&db);
    v2_store
        .decompose_cospan(
            &Cospan::new(vec![0], vec![0], vec!['c']),
            "isolated",
            serde_json::json!({}),
            |c| c.to_string(),
        )
        .await
        .unwrap();

    // V1 sees only its 2 records
    let v1_ids = v1_store.list().await.unwrap();
    assert_eq!(v1_ids.len(), 2, "V1 should have exactly 2 cospan records");

    // V2 node_store sees only V2 nodes (not V1 records)
    let node_store = NodeStore::new(&db);
    let v2_nodes = node_store.list().await.unwrap();
    // decompose_cospan creates 1 middle node for a single-element cospan
    assert_eq!(v2_nodes.len(), 1, "V2 should have exactly 1 graph_node");
}
