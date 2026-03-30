use catgraph::span::Span;
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

#[tokio::test]
async fn test_span_roundtrip_identity() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    // Identity span: left == right, middle is diagonal
    let span = Span::new(
        vec!['a', 'b', 'c'],
        vec!['a', 'b', 'c'],
        vec![(0, 0), (1, 1), (2, 2)],
    );
    assert!(span.is_left_identity());
    assert!(span.is_right_identity());

    let hub_id = store
        .decompose_span(&span, "identity", serde_json::json!({}), |c| c.to_string())
        .await
        .unwrap();

    let reconstructed: Span<char> = store.reconstruct_span(&hub_id).await.unwrap();

    assert_eq!(reconstructed.left(), span.left());
    assert_eq!(reconstructed.right(), span.right());
    assert_eq!(reconstructed.middle_pairs(), span.middle_pairs());
    assert!(reconstructed.is_left_identity());
    assert!(reconstructed.is_right_identity());
}

#[tokio::test]
async fn test_span_roundtrip_non_identity() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    // Non-identity: two left elements map to the same right element
    let span = Span::new(vec!['a', 'a'], vec!['a'], vec![(0, 0), (1, 0)]);
    assert!(span.is_left_identity());
    assert!(!span.is_right_identity());

    let hub_id = store
        .decompose_span(&span, "non_id", serde_json::json!({}), |c| c.to_string())
        .await
        .unwrap();

    let reconstructed: Span<char> = store.reconstruct_span(&hub_id).await.unwrap();

    assert_eq!(reconstructed.left(), span.left());
    assert_eq!(reconstructed.right(), span.right());
    assert_eq!(reconstructed.middle_pairs(), span.middle_pairs());
    assert!(reconstructed.is_left_identity());
    assert!(!reconstructed.is_right_identity());
}

#[tokio::test]
async fn test_span_roundtrip_non_square() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    // |left| = 3, |right| = 2 — non-square
    let span = Span::new(
        vec!['x', 'y', 'x'],
        vec!['x', 'y'],
        vec![(0, 0), (1, 1), (2, 0)],
    );

    let hub_id = store
        .decompose_span(&span, "non_square", serde_json::json!({}), |c| c.to_string())
        .await
        .unwrap();

    let reconstructed: Span<char> = store.reconstruct_span(&hub_id).await.unwrap();

    assert_eq!(reconstructed.left(), span.left());
    assert_eq!(reconstructed.right(), span.right());
    assert_eq!(reconstructed.middle_pairs(), span.middle_pairs());

    // Verify hub metadata counts
    let hub = store.get_hub(&hub_id).await.unwrap();
    assert_eq!(hub.source_count, 3);
    assert_eq!(hub.target_count, 2);
}

#[tokio::test]
async fn test_span_roundtrip_u32_labels() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    let span = Span::new(vec![10u32, 20, 10], vec![10u32, 20], vec![(0, 0), (1, 1), (2, 0)]);

    let hub_id = store
        .decompose_span(&span, "u32_span", serde_json::json!({}), |n| n.to_string())
        .await
        .unwrap();

    let reconstructed: Span<u32> = store.reconstruct_span(&hub_id).await.unwrap();

    assert_eq!(reconstructed.left(), span.left());
    assert_eq!(reconstructed.right(), span.right());
    assert_eq!(reconstructed.middle_pairs(), span.middle_pairs());
}

#[tokio::test]
async fn test_reconstruct_span_missing_middle_pairs() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    // Decompose a cospan — its hub has no middle_pairs property
    let cospan = catgraph::cospan::Cospan::new(vec![0], vec![0], vec!['z']);
    let hub_id = store
        .decompose_cospan(&cospan, "not_a_span", serde_json::json!({}), |c| {
            c.to_string()
        })
        .await
        .unwrap();

    // Attempting to reconstruct_span on a cospan hub should fail
    let result: Result<Span<char>, _> = store.reconstruct_span(&hub_id).await;
    assert!(result.is_err(), "should fail when middle_pairs is missing");
}
