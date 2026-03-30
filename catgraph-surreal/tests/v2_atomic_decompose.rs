use catgraph::cospan::Cospan;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

use catgraph_surreal::hyperedge_store::HyperedgeStore;
use catgraph_surreal::init_schema_v2;

async fn setup_v2() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    init_schema_v2(&db).await.unwrap();
    db
}

// ---------------------------------------------------------------------------
// 1. Atomic decompose matches non-atomic
// ---------------------------------------------------------------------------

#[tokio::test]
async fn atomic_decompose_matches_non_atomic() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    // Cospan: {a,b} -> {x,y,z} <- {c}
    let cospan = Cospan::new(vec![0, 1], vec![2], vec!['x', 'y', 'z']);

    // Decompose atomically
    let hub_id = store
        .decompose_cospan_atomic(&cospan, "atomic_test", serde_json::json!({}), |c| {
            c.to_string()
        })
        .await
        .unwrap();

    // Reconstruct and verify structural equality with the original cospan
    let reconstructed: Cospan<char> = store.reconstruct_cospan(&hub_id).await.unwrap();

    assert_eq!(reconstructed.left_to_middle(), cospan.left_to_middle());
    assert_eq!(reconstructed.right_to_middle(), cospan.right_to_middle());
    assert_eq!(reconstructed.middle(), cospan.middle());
}

#[tokio::test]
async fn atomic_decompose_hub_metadata() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    let cospan = Cospan::new(vec![0, 1], vec![2], vec!['x', 'y', 'z']);

    let hub_id = store
        .decompose_cospan_atomic(&cospan, "meta_test", serde_json::json!({"key": "val"}), |c| {
            c.to_string()
        })
        .await
        .unwrap();

    let hub = store.get_hub(&hub_id).await.unwrap();
    assert_eq!(hub.kind, "meta_test");
    assert_eq!(hub.source_count, 2);
    assert_eq!(hub.target_count, 1);
    assert_eq!(hub.properties["key"], "val");
}

// ---------------------------------------------------------------------------
// 2. Identity cospan (single middle node, both sides map to it)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn atomic_decompose_identity() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    // Identity: both left and right map to the same single middle node
    let cospan = Cospan::new(vec![0, 0], vec![0], vec!['m']);

    let hub_id = store
        .decompose_cospan_atomic(&cospan, "identity", serde_json::json!({}), |c| {
            c.to_string()
        })
        .await
        .unwrap();

    let reconstructed: Cospan<char> = store.reconstruct_cospan(&hub_id).await.unwrap();

    assert_eq!(reconstructed.left_to_middle(), &[0, 0]);
    assert_eq!(reconstructed.right_to_middle(), &[0]);
    assert_eq!(reconstructed.middle(), &['m']);
}

// ---------------------------------------------------------------------------
// 3. Larger cospan (5+ middle nodes)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn atomic_decompose_many_nodes() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    // 5 middle nodes, left maps [0,1,2,3], right maps [2,3,4]
    let cospan = Cospan::new(
        vec![0, 1, 2, 3],
        vec![2, 3, 4],
        vec!['a', 'b', 'c', 'd', 'e'],
    );

    let hub_id = store
        .decompose_cospan_atomic(&cospan, "large", serde_json::json!({}), |c| c.to_string())
        .await
        .unwrap();

    // Verify sources and targets
    let sources = store.sources(&hub_id).await.unwrap();
    assert_eq!(sources.len(), 4, "expected 4 source nodes");

    let targets = store.targets(&hub_id).await.unwrap();
    assert_eq!(targets.len(), 3, "expected 3 target nodes");

    // Reconstruct and verify full structural equality
    let reconstructed: Cospan<char> = store.reconstruct_cospan(&hub_id).await.unwrap();
    assert_eq!(reconstructed.left_to_middle(), cospan.left_to_middle());
    assert_eq!(reconstructed.right_to_middle(), cospan.right_to_middle());
    assert_eq!(reconstructed.middle(), cospan.middle());
}

// ---------------------------------------------------------------------------
// 4. Both atomic and non-atomic produce consistent reconstruction
// ---------------------------------------------------------------------------

#[tokio::test]
async fn atomic_vs_non_atomic_structural_equivalence() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    let cospan = Cospan::new(vec![0, 1, 0], vec![1, 2], vec!['p', 'q', 'r']);

    // Non-atomic decompose
    let hub_non_atomic = store
        .decompose_cospan(&cospan, "non_atomic", serde_json::json!({}), |c| c.to_string())
        .await
        .unwrap();
    let recon_non_atomic: Cospan<char> =
        store.reconstruct_cospan(&hub_non_atomic).await.unwrap();

    // Atomic decompose
    let hub_atomic = store
        .decompose_cospan_atomic(&cospan, "atomic", serde_json::json!({}), |c| c.to_string())
        .await
        .unwrap();
    let recon_atomic: Cospan<char> = store.reconstruct_cospan(&hub_atomic).await.unwrap();

    // Both reconstructions must match the original
    assert_eq!(recon_non_atomic.left_to_middle(), cospan.left_to_middle());
    assert_eq!(recon_non_atomic.right_to_middle(), cospan.right_to_middle());
    assert_eq!(recon_non_atomic.middle(), cospan.middle());

    assert_eq!(recon_atomic.left_to_middle(), cospan.left_to_middle());
    assert_eq!(recon_atomic.right_to_middle(), cospan.right_to_middle());
    assert_eq!(recon_atomic.middle(), cospan.middle());
}

// ---------------------------------------------------------------------------
// 5. Retry wrapper (no conflict, should succeed on first attempt)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn retry_wrapper_succeeds_immediately() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    let cospan = Cospan::new(vec![0], vec![0], vec!['z']);

    let hub_id = store
        .decompose_cospan_with_retry(
            &cospan,
            "retry_test",
            serde_json::json!({}),
            |c| c.to_string(),
            3,
        )
        .await
        .unwrap();

    let reconstructed: Cospan<char> = store.reconstruct_cospan(&hub_id).await.unwrap();
    assert_eq!(reconstructed.middle(), &['z']);
}

// ---------------------------------------------------------------------------
// 6. PersistError::is_transaction_conflict
// ---------------------------------------------------------------------------

#[tokio::test]
async fn error_detection_non_conflict() {
    use catgraph_surreal::error::PersistError;

    let not_conflict = PersistError::InvalidData("test".into());
    assert!(!not_conflict.is_transaction_conflict());

    let not_found = PersistError::NotFound("missing".into());
    assert!(!not_found.is_transaction_conflict());

    let conflict = PersistError::TransactionConflict("write collision".into());
    assert!(conflict.is_transaction_conflict());
}

// ---------------------------------------------------------------------------
// 7. Atomic decompose with u32 labels
// ---------------------------------------------------------------------------

#[tokio::test]
async fn atomic_decompose_u32_labels() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    let cospan = Cospan::new(vec![0, 1], vec![1, 2], vec![10u32, 20, 30]);

    let hub_id = store
        .decompose_cospan_atomic(&cospan, "u32_test", serde_json::json!({}), |n| {
            format!("node_{n}")
        })
        .await
        .unwrap();

    let reconstructed: Cospan<u32> = store.reconstruct_cospan(&hub_id).await.unwrap();
    assert_eq!(reconstructed.left_to_middle(), &[0, 1]);
    assert_eq!(reconstructed.right_to_middle(), &[1, 2]);
    assert_eq!(reconstructed.middle(), &[10, 20, 30]);
}
