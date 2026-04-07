use catgraph::cospan::Cospan;
use catgraph::named_cospan::NamedCospan;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

use catgraph_surreal::hyperedge_store::HyperedgeStore;
use catgraph_surreal::init_schema_v2;

async fn setup() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    init_schema_v2(&db).await.unwrap();
    db
}

#[tokio::test]
async fn named_cospan_roundtrip_via_hub() {
    let db = setup().await;
    let store = HyperedgeStore::new(&db);

    // 2 left ports -> middle {x, y}, 1 right port -> middle y
    let nc = NamedCospan::<char, String, String>::new(
        vec![0, 1],
        vec![1],
        vec!['x', 'y'],
        vec!["input_a".into(), "input_b".into()],
        vec!["output_c".into()],
    );

    let hub_id = store
        .decompose_named_cospan(&nc, "named_test", serde_json::json!({}))
        .await
        .unwrap();

    let reconstructed: NamedCospan<char, String, String> =
        store.reconstruct_named_cospan(&hub_id).await.unwrap();

    assert_eq!(
        reconstructed.cospan().left_to_middle(),
        nc.cospan().left_to_middle()
    );
    assert_eq!(
        reconstructed.cospan().right_to_middle(),
        nc.cospan().right_to_middle()
    );
    assert_eq!(reconstructed.cospan().middle(), nc.cospan().middle());
    assert_eq!(reconstructed.left_names(), nc.left_names());
    assert_eq!(reconstructed.right_names(), nc.right_names());
}

#[tokio::test]
async fn named_cospan_empty_roundtrip() {
    let db = setup().await;
    let store = HyperedgeStore::new(&db);

    let nc = NamedCospan::<char, String, String>::empty();

    let hub_id = store
        .decompose_named_cospan(&nc, "empty_nc", serde_json::json!({}))
        .await
        .unwrap();

    let reconstructed: NamedCospan<char, String, String> =
        store.reconstruct_named_cospan(&hub_id).await.unwrap();

    assert!(reconstructed.cospan().left_to_middle().is_empty());
    assert!(reconstructed.cospan().right_to_middle().is_empty());
    assert!(reconstructed.cospan().middle().is_empty());
    assert!(reconstructed.left_names().is_empty());
    assert!(reconstructed.right_names().is_empty());
}

#[tokio::test]
async fn named_cospan_port_names_preserved_with_shared_middle() {
    let db = setup().await;
    let store = HyperedgeStore::new(&db);

    // 3 left ports mapping to 2 middle elements (indices 0, 1, 0 — shared)
    let nc = NamedCospan::<char, String, String>::new(
        vec![0, 1, 0],
        vec![1],
        vec!['a', 'b'],
        vec!["port_x".into(), "port_y".into(), "port_z".into()],
        vec!["out_w".into()],
    );

    let hub_id = store
        .decompose_named_cospan(&nc, "shared_middle", serde_json::json!({}))
        .await
        .unwrap();

    let reconstructed: NamedCospan<char, String, String> =
        store.reconstruct_named_cospan(&hub_id).await.unwrap();

    // Port name ordering must be preserved exactly
    assert_eq!(
        reconstructed.left_names(),
        &vec![
            "port_x".to_string(),
            "port_y".to_string(),
            "port_z".to_string()
        ]
    );
    assert_eq!(reconstructed.right_names(), &vec!["out_w".to_string()]);

    // Structural maps must match
    assert_eq!(
        reconstructed.cospan().left_to_middle(),
        nc.cospan().left_to_middle()
    );
    assert_eq!(
        reconstructed.cospan().right_to_middle(),
        nc.cospan().right_to_middle()
    );
    assert_eq!(reconstructed.cospan().middle(), nc.cospan().middle());
}

#[tokio::test]
async fn named_cospan_hub_properties_merge() {
    let db = setup().await;
    let store = HyperedgeStore::new(&db);

    let nc = NamedCospan::<char, String, String>::new(
        vec![0],
        vec![0],
        vec!['q'],
        vec!["in_1".into()],
        vec!["out_1".into()],
    );

    let custom_props = serde_json::json!({
        "custom_key": "custom_value",
        "version": 42,
    });

    let hub_id = store
        .decompose_named_cospan(&nc, "merged_props", custom_props)
        .await
        .unwrap();

    // Verify both port names and custom keys exist in hub properties
    let hub = store.get_hub(&hub_id).await.unwrap();
    assert_eq!(hub.properties["custom_key"], "custom_value");
    assert_eq!(hub.properties["version"], 42);
    assert_eq!(
        hub.properties["left_port_names"],
        serde_json::json!(["in_1"])
    );
    assert_eq!(
        hub.properties["right_port_names"],
        serde_json::json!(["out_1"])
    );
}

#[tokio::test]
async fn reconstruct_plain_cospan_hub_errors_gracefully() {
    let db = setup().await;
    let store = HyperedgeStore::new(&db);

    // Decompose a plain Cospan — no port names in properties
    let cospan = Cospan::new(vec![0, 1], vec![1], vec!['a', 'b']);
    let hub_id = store
        .decompose_cospan(&cospan, "plain", serde_json::json!({}), |c| c.to_string())
        .await
        .unwrap();

    // Attempting reconstruct_named_cospan should fail with InvalidData
    let result: Result<NamedCospan<char, String, String>, _> =
        store.reconstruct_named_cospan(&hub_id).await;

    match result {
        Ok(_) => panic!("should fail when port names are missing"),
        Err(err) => {
            let err_msg = format!("{err}");
            assert!(
                err_msg.contains("left_port_names"),
                "error should mention missing left_port_names, got: {err_msg}"
            );
        }
    }
}
