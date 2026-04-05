use surrealdb::engine::local::Mem;
use surrealdb::types::RecordId;
use surrealdb::Surreal;

use catgraph_surreal::edge_store::EdgeStore;
use catgraph_surreal::init_schema_v2;
use catgraph_surreal::node_store::NodeStore;
use catgraph_surreal::query::QueryHelper;

async fn setup() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    init_schema_v2(&db).await.unwrap();
    db
}

/// Build a linear chain: a -> b -> c -> d
async fn build_chain(db: &Surreal<surrealdb::engine::local::Db>) -> Vec<RecordId> {
    let ns = NodeStore::new(db);
    let es = EdgeStore::new(db);
    let a = ns
        .create("a", "node", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let b = ns
        .create("b", "node", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let c = ns
        .create("c", "node", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let d = ns
        .create("d", "node", vec![], serde_json::json!({}))
        .await
        .unwrap();
    es.relate(&a, &b, "flow", None, serde_json::json!({}))
        .await
        .unwrap();
    es.relate(&b, &c, "flow", None, serde_json::json!({}))
        .await
        .unwrap();
    es.relate(&c, &d, "flow", None, serde_json::json!({}))
        .await
        .unwrap();
    vec![a, b, c, d]
}

// ---------------------------------------------------------------------------
// shortest_path
// ---------------------------------------------------------------------------

#[tokio::test]
async fn shortest_path_linear_chain() {
    let db = setup().await;
    let ids = build_chain(&db).await;
    let qh = QueryHelper::new(&db);

    let path = qh
        .shortest_path(&ids[0], &ids[3], "flow", 10)
        .await
        .unwrap();
    assert!(path.is_some());
    let path = path.unwrap();
    assert_eq!(path.len(), 4); // a -> b -> c -> d
    assert_eq!(path[0].name, "a");
    assert_eq!(path[1].name, "b");
    assert_eq!(path[2].name, "c");
    assert_eq!(path[3].name, "d");
}

#[tokio::test]
async fn shortest_path_not_reachable() {
    let db = setup().await;
    let ids = build_chain(&db).await;
    let qh = QueryHelper::new(&db);

    // d -> a is not reachable (directed edges)
    let path = qh
        .shortest_path(&ids[3], &ids[0], "flow", 10)
        .await
        .unwrap();
    assert!(path.is_none());
}

#[tokio::test]
async fn shortest_path_same_node() {
    let db = setup().await;
    let ids = build_chain(&db).await;
    let qh = QueryHelper::new(&db);

    let path = qh
        .shortest_path(&ids[0], &ids[0], "flow", 10)
        .await
        .unwrap();
    assert!(path.is_some());
    let path = path.unwrap();
    assert_eq!(path.len(), 1);
    assert_eq!(path[0].name, "a");
}

#[tokio::test]
async fn shortest_path_depth_limit_prevents_discovery() {
    let db = setup().await;
    let ids = build_chain(&db).await;
    let qh = QueryHelper::new(&db);

    // a -> d requires 3 hops; max_depth=2 should fail
    let path = qh
        .shortest_path(&ids[0], &ids[3], "flow", 2)
        .await
        .unwrap();
    assert!(path.is_none());
}

#[tokio::test]
async fn shortest_path_picks_shorter_route() {
    let db = setup().await;
    let ns = NodeStore::new(&db);
    let es = EdgeStore::new(&db);

    // Build diamond: a -> b -> d and a -> c -> d
    let a = ns
        .create("a", "node", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let b = ns
        .create("b", "node", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let c = ns
        .create("c", "node", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let d = ns
        .create("d", "node", vec![], serde_json::json!({}))
        .await
        .unwrap();
    es.relate(&a, &b, "flow", None, serde_json::json!({}))
        .await
        .unwrap();
    es.relate(&a, &c, "flow", None, serde_json::json!({}))
        .await
        .unwrap();
    es.relate(&b, &d, "flow", None, serde_json::json!({}))
        .await
        .unwrap();
    es.relate(&c, &d, "flow", None, serde_json::json!({}))
        .await
        .unwrap();

    let qh = QueryHelper::new(&db);
    let path = qh.shortest_path(&a, &d, "flow", 10).await.unwrap();
    assert!(path.is_some());
    let path = path.unwrap();
    // Both routes are 2 hops; BFS finds one of them
    assert_eq!(path.len(), 3); // a -> (b|c) -> d
    assert_eq!(path[0].name, "a");
    assert_eq!(path[2].name, "d");
}

#[tokio::test]
async fn shortest_path_wrong_edge_kind() {
    let db = setup().await;
    let ids = build_chain(&db).await;
    let qh = QueryHelper::new(&db);

    // Edges are "flow" kind, querying "data" should find nothing
    let path = qh
        .shortest_path(&ids[0], &ids[3], "data", 10)
        .await
        .unwrap();
    assert!(path.is_none());
}

// ---------------------------------------------------------------------------
// collect_reachable
// ---------------------------------------------------------------------------

#[tokio::test]
async fn collect_reachable_all() {
    let db = setup().await;
    let ids = build_chain(&db).await;
    let qh = QueryHelper::new(&db);

    let all = qh.collect_reachable(&ids[0], "flow", 10).await.unwrap();
    assert_eq!(all.len(), 3); // b, c, d (excludes start)
}

#[tokio::test]
async fn collect_reachable_depth_limited() {
    let db = setup().await;
    let ids = build_chain(&db).await;
    let qh = QueryHelper::new(&db);

    let near = qh.collect_reachable(&ids[0], "flow", 1).await.unwrap();
    assert_eq!(near.len(), 1); // only b
    assert_eq!(near[0].name, "b");
}

#[tokio::test]
async fn collect_reachable_leaf_node() {
    let db = setup().await;
    let ids = build_chain(&db).await;
    let qh = QueryHelper::new(&db);

    // d is a leaf — nothing reachable from it
    let leaf = qh.collect_reachable(&ids[3], "flow", 10).await.unwrap();
    assert!(leaf.is_empty());
}

#[tokio::test]
async fn collect_reachable_matches_reachable() {
    let db = setup().await;
    let ids = build_chain(&db).await;
    let qh = QueryHelper::new(&db);

    let via_reachable = qh.reachable(&ids[0], "flow", 10).await.unwrap();
    let via_collect = qh.collect_reachable(&ids[0], "flow", 10).await.unwrap();
    assert_eq!(via_reachable.len(), via_collect.len());
    for (a, b) in via_reachable.iter().zip(via_collect.iter()) {
        assert_eq!(a.name, b.name);
    }
}
