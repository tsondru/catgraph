//! Use Case: Parallel Computing / Dataflow
//!
//! Models dataflow operations (e.g., matrix multiply) using `NamedCospan` with
//! port names. Demonstrates V2 decomposition of named cospans alongside V1
//! `NamedCospanStore` persistence, verifying both coexist.

use catgraph::named_cospan::NamedCospan;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

use catgraph_surreal::hyperedge_store::HyperedgeStore;
use catgraph_surreal::named_cospan_store::NamedCospanStore;
use catgraph_surreal::{init_schema, init_schema_v2};

async fn setup_both() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    init_schema(&db).await.unwrap();
    init_schema_v2(&db).await.unwrap();
    db
}

// ---------------------------------------------------------------------------
// 1. Matrix multiply: named ports decomposed to V2
// ---------------------------------------------------------------------------

#[tokio::test]
async fn matrix_multiply_named_cospan_v2() {
    let db = setup_both().await;
    let v2 = HyperedgeStore::new(&db);

    // NamedCospan: left=[Matrix_A, Matrix_B] → middle → right=[Result]
    // middle elements represent the operation nodes
    // left_map=[0, 1], right_map=[2], middle=['A', 'B', 'R']
    let nc: NamedCospan<char, String, String> = NamedCospan::new(
        vec![0, 1],                                              // left_map
        vec![2],                                                 // right_map
        vec!['A', 'B', 'R'],                                    // middle
        vec!["Matrix_A".to_string(), "Matrix_B".to_string()],   // left_names
        vec!["Result".to_string()],                              // right_names
    );

    let hub_id = v2
        .decompose_named_cospan(&nc, "matrix_multiply", serde_json::json!({"algorithm": "strassen"}))
        .await
        .unwrap();

    // Sources = inputs = 2
    let sources = v2.sources(&hub_id).await.unwrap();
    assert_eq!(sources.len(), 2);

    // Targets = outputs = 1
    let targets = v2.targets(&hub_id).await.unwrap();
    assert_eq!(targets.len(), 1);

    // Hub metadata
    let hub = v2.get_hub(&hub_id).await.unwrap();
    assert_eq!(hub.kind, "matrix_multiply");
    assert_eq!(hub.properties["algorithm"], "strassen");
}

// ---------------------------------------------------------------------------
// 2. V1 + V2 coexistence for the same NamedCospan
// ---------------------------------------------------------------------------

#[tokio::test]
async fn named_cospan_v1_v2_coexistence() {
    let db = setup_both().await;
    let v1 = NamedCospanStore::new(&db);
    let v2 = HyperedgeStore::new(&db);

    let nc: NamedCospan<char, String, String> = NamedCospan::new(
        vec![0, 1],
        vec![2],
        vec!['A', 'B', 'R'],
        vec!["input_a".to_string(), "input_b".to_string()],
        vec!["output".to_string()],
    );

    // Save via V1
    let v1_id = v1.save(&nc).await.unwrap();

    // Decompose via V2
    let v2_hub = v2
        .decompose_named_cospan(&nc, "multiply", serde_json::json!({}))
        .await
        .unwrap();

    // V1 load still works — roundtrip preserves structure
    let v1_loaded: NamedCospan<char, String, String> = v1.load(&v1_id).await.unwrap();
    assert_eq!(v1_loaded.cospan().left_to_middle(), nc.cospan().left_to_middle());
    assert_eq!(v1_loaded.cospan().right_to_middle(), nc.cospan().right_to_middle());
    assert_eq!(v1_loaded.cospan().middle(), nc.cospan().middle());
    assert_eq!(v1_loaded.left_names(), nc.left_names());
    assert_eq!(v1_loaded.right_names(), nc.right_names());

    // V2 reconstruct also works (underlying cospan)
    let v2_cospan: catgraph::cospan::Cospan<char> =
        v2.reconstruct_cospan(&v2_hub).await.unwrap();
    assert_eq!(v2_cospan.left_to_middle(), nc.cospan().left_to_middle());
    assert_eq!(v2_cospan.right_to_middle(), nc.cospan().right_to_middle());
    assert_eq!(v2_cospan.middle(), nc.cospan().middle());
}

// ---------------------------------------------------------------------------
// 3. Pipeline: map → reduce → collect (chained operations)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dataflow_pipeline_three_stages() {
    let db = setup_both().await;
    let v2 = HyperedgeStore::new(&db);

    // Stage 1: map — 3 inputs, 3 outputs
    let map_op: NamedCospan<i32, String, String> = NamedCospan::new(
        vec![0, 1, 2],
        vec![0, 1, 2],
        vec![0, 1, 2],
        vec!["chunk_0".into(), "chunk_1".into(), "chunk_2".into()],
        vec!["mapped_0".into(), "mapped_1".into(), "mapped_2".into()],
    );

    let map_hub = v2
        .decompose_named_cospan(
            &map_op,
            "map",
            serde_json::json!({"parallelism": 3}),
        )
        .await
        .unwrap();

    // Stage 2: reduce — 3 inputs, 1 output
    let reduce_op: NamedCospan<i32, String, String> = NamedCospan::new(
        vec![0, 1, 2],
        vec![3],
        vec![0, 1, 2, 3],
        vec!["mapped_0".into(), "mapped_1".into(), "mapped_2".into()],
        vec!["reduced".into()],
    );

    let reduce_hub = v2
        .decompose_named_cospan(
            &reduce_op,
            "reduce",
            serde_json::json!({"combiner": "sum"}),
        )
        .await
        .unwrap();

    // Verify stage shapes
    let map_meta = v2.get_hub(&map_hub).await.unwrap();
    assert_eq!(map_meta.source_count, 3);
    assert_eq!(map_meta.target_count, 3);

    let reduce_meta = v2.get_hub(&reduce_hub).await.unwrap();
    assert_eq!(reduce_meta.source_count, 3);
    assert_eq!(reduce_meta.target_count, 1);
}

// ---------------------------------------------------------------------------
// 4. Fan-out: broadcast one input to many workers
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dataflow_broadcast_fan_out() {
    let db = setup_both().await;
    let v2 = HyperedgeStore::new(&db);

    // Broadcast: 1 input → 4 outputs (same middle node for the source)
    // middle = [data=0, w0=1, w1=2, w2=3, w3=4]
    // left_map = [0]  (single source)
    // right_map = [1, 2, 3, 4]  (4 workers)
    let broadcast: NamedCospan<i32, String, String> = NamedCospan::new(
        vec![0],
        vec![1, 2, 3, 4],
        vec![0, 1, 2, 3, 4],
        vec!["data_stream".into()],
        vec!["worker_0".into(), "worker_1".into(), "worker_2".into(), "worker_3".into()],
    );

    let hub_id = v2
        .decompose_named_cospan(
            &broadcast,
            "broadcast",
            serde_json::json!({"strategy": "round_robin"}),
        )
        .await
        .unwrap();

    let hub = v2.get_hub(&hub_id).await.unwrap();
    assert_eq!(hub.source_count, 1);
    assert_eq!(hub.target_count, 4);

    let targets = v2.targets(&hub_id).await.unwrap();
    assert_eq!(targets.len(), 4);
}
