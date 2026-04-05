use catgraph_surreal::{
    edge_store::EdgeStore, fingerprint::FingerprintEngine, init_schema_v2,
    node_store::NodeStore,
};
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

async fn setup() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    init_schema_v2(&db).await.unwrap();
    db
}

#[tokio::test]
async fn compute_fingerprint_basic() {
    let db = setup().await;
    let ns = NodeStore::new(&db);
    let es = EdgeStore::new(&db);
    let fe = FingerprintEngine::new(&db, 8);

    let a = ns
        .create("hub_node", "node", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let b = ns
        .create("leaf1", "node", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let c = ns
        .create("leaf2", "node", vec![], serde_json::json!({}))
        .await
        .unwrap();
    es.relate(&a, &b, "flow", None, serde_json::json!({}))
        .await
        .unwrap();
    es.relate(&a, &c, "flow", None, serde_json::json!({}))
        .await
        .unwrap();

    let fp = fe.compute_fingerprint(&a).await.unwrap();
    assert_eq!(fp.len(), 8);
    assert_eq!(fp[0], 2.0); // out-degree = 2
    assert_eq!(fp[1], 0.0); // in-degree = 0
    assert_eq!(fp[2], 2.0); // total degree = 2
}

#[tokio::test]
async fn store_and_retrieve_fingerprint() {
    let db = setup().await;
    let ns = NodeStore::new(&db);
    let fe = FingerprintEngine::new(&db, 4);

    let a = ns
        .create("a", "node", vec![], serde_json::json!({}))
        .await
        .unwrap();
    fe.store_fingerprint(&a, &[1.0, 2.0, 3.0, 4.0])
        .await
        .unwrap();

    let node = ns.get(&a).await.unwrap();
    assert!(node.embedding.is_some());
    assert_eq!(node.embedding.unwrap(), vec![1.0, 2.0, 3.0, 4.0]);
}

#[tokio::test]
async fn index_node_computes_and_stores() {
    let db = setup().await;
    let ns = NodeStore::new(&db);
    let fe = FingerprintEngine::new(&db, 8);

    let a = ns
        .create("a", "node", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let fp = fe.index_node(&a).await.unwrap();
    assert_eq!(fp.len(), 8);

    let node = ns.get(&a).await.unwrap();
    assert!(node.embedding.is_some());
}

#[tokio::test]
async fn hnsw_search_finds_similar() {
    let db = setup().await;
    let ns = NodeStore::new(&db);
    let fe = FingerprintEngine::new(&db, 4);
    fe.init_index().await.unwrap();

    let a = ns
        .create("exact_match", "node", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let b = ns
        .create("close_match", "node", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let c = ns
        .create("far_away", "node", vec![], serde_json::json!({}))
        .await
        .unwrap();

    fe.store_fingerprint(&a, &[1.0, 2.0, 3.0, 0.0])
        .await
        .unwrap();
    fe.store_fingerprint(&b, &[1.0, 2.0, 3.1, 0.0])
        .await
        .unwrap();
    fe.store_fingerprint(&c, &[100.0, 200.0, 300.0, 0.0])
        .await
        .unwrap();

    let results = fe
        .search_similar(&[1.0, 2.0, 3.0, 0.0], 3, 50)
        .await
        .unwrap();
    assert!(!results.is_empty());
    // First result should be exact_match (distance 0 or near-zero)
    assert_eq!(results[0].0.name, "exact_match");
    // Second should be close_match
    assert_eq!(results[1].0.name, "close_match");
}

#[tokio::test]
async fn isolated_node_fingerprint_is_zero() {
    let db = setup().await;
    let ns = NodeStore::new(&db);
    let fe = FingerprintEngine::new(&db, 4);

    let a = ns
        .create("isolated", "node", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let fp = fe.compute_fingerprint(&a).await.unwrap();
    assert!(fp.iter().all(|x| *x == 0.0));
}
