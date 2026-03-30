//! Use Case: Circuit Design
//!
//! Models logic gates as hyperedges. Demonstrates composition through shared
//! nodes (output of one gate feeds input of another) and both V1 + V2
//! persistence of the same circuit.

use catgraph::cospan::Cospan;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

use catgraph_surreal::cospan_store::CospanStore;
use catgraph_surreal::hyperedge_store::HyperedgeStore;
use catgraph_surreal::{init_schema, init_schema_v2};

async fn setup_both() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    init_schema(&db).await.unwrap();
    init_schema_v2(&db).await.unwrap();
    db
}

fn gate_name<'a>(names: &'a [&'a str]) -> impl Fn(&i32) -> String + 'a {
    move |i: &i32| names[usize::try_from(*i).expect("non-negative index")].to_string()
}

// ---------------------------------------------------------------------------
// 1. AND gate: Input_A, Input_B → Output
// ---------------------------------------------------------------------------

#[tokio::test]
async fn and_gate_decompose() {
    let db = setup_both().await;
    let v2 = HyperedgeStore::new(&db);

    // AND: sources=[Input_A, Input_B] → targets=[Output]
    // middle = [A=0, B=1, Out=2]
    let and_gate: Cospan<i32> = Cospan::new(vec![0, 1], vec![2], vec![0, 1, 2]);
    let names = ["Input_A", "Input_B", "AND_Output"];

    let hub_id = v2
        .decompose_cospan(
            &and_gate,
            "AND",
            serde_json::json!({"gate_type": "AND", "delay_ns": 2}),
            gate_name(&names),
        )
        .await
        .unwrap();

    let hub = v2.get_hub(&hub_id).await.unwrap();
    assert_eq!(hub.kind, "AND");
    assert_eq!(hub.source_count, 2);
    assert_eq!(hub.target_count, 1);
    assert_eq!(hub.properties["delay_ns"], 2);

    let inputs = v2.sources(&hub_id).await.unwrap();
    assert_eq!(inputs.len(), 2);

    let outputs = v2.targets(&hub_id).await.unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].name, "AND_Output");
}

// ---------------------------------------------------------------------------
// 2. Cascaded gates: AND → OR (composition through shared output)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cascaded_and_or_gates() {
    let db = setup_both().await;
    let v2 = HyperedgeStore::new(&db);

    // AND gate: [Input_A, Input_B] → [AND_Out]
    let and_gate: Cospan<i32> = Cospan::new(vec![0, 1], vec![2], vec![0, 1, 2]);
    let and_names = ["Input_A", "Input_B", "AND_Out"];

    let and_hub = v2
        .decompose_cospan(
            &and_gate,
            "AND",
            serde_json::json!({"delay_ns": 2}),
            gate_name(&and_names),
        )
        .await
        .unwrap();

    // OR gate: [AND_Out, Input_C] → [Final]
    // AND_Out feeds into the OR gate as a source
    let or_gate: Cospan<i32> = Cospan::new(vec![0, 1], vec![2], vec![0, 1, 2]);
    let or_names = ["AND_Out", "Input_C", "Final"];

    let or_hub = v2
        .decompose_cospan(
            &or_gate,
            "OR",
            serde_json::json!({"delay_ns": 1}),
            gate_name(&or_names),
        )
        .await
        .unwrap();

    // AND gate: 2 inputs, 1 output
    let and_inputs = v2.sources(&and_hub).await.unwrap();
    assert_eq!(and_inputs.len(), 2);
    let and_outputs = v2.targets(&and_hub).await.unwrap();
    assert_eq!(and_outputs.len(), 1);
    assert_eq!(and_outputs[0].name, "AND_Out");

    // OR gate: 2 inputs (including AND_Out), 1 output
    let or_inputs = v2.sources(&or_hub).await.unwrap();
    assert_eq!(or_inputs.len(), 2);
    let or_input_names: Vec<&str> = or_inputs.iter().map(|n| n.name.as_str()).collect();
    assert!(or_input_names.contains(&"AND_Out"));
    assert!(or_input_names.contains(&"Input_C"));

    let or_outputs = v2.targets(&or_hub).await.unwrap();
    assert_eq!(or_outputs.len(), 1);
    assert_eq!(or_outputs[0].name, "Final");
}

// ---------------------------------------------------------------------------
// 3. V1 + V2 roundtrip of the same gate
// ---------------------------------------------------------------------------

#[tokio::test]
async fn gate_v1_v2_roundtrip() {
    let db = setup_both().await;
    let v1 = CospanStore::new(&db);
    let v2 = HyperedgeStore::new(&db);

    let and_gate: Cospan<i32> = Cospan::new(vec![0, 1], vec![2], vec![0, 1, 2]);
    let names = ["A", "B", "Out"];

    // V1 save
    let v1_id = v1.save(&and_gate).await.unwrap();

    // V2 decompose
    let v2_hub = v2
        .decompose_cospan(&and_gate, "AND", serde_json::json!({}), gate_name(&names))
        .await
        .unwrap();

    // V1 roundtrip
    let v1_loaded: Cospan<i32> = v1.load(&v1_id).await.unwrap();
    assert_eq!(v1_loaded.left_to_middle(), and_gate.left_to_middle());
    assert_eq!(v1_loaded.right_to_middle(), and_gate.right_to_middle());
    assert_eq!(v1_loaded.middle(), and_gate.middle());

    // V2 roundtrip
    let v2_loaded: Cospan<i32> = v2.reconstruct_cospan(&v2_hub).await.unwrap();
    assert_eq!(v2_loaded.left_to_middle(), and_gate.left_to_middle());
    assert_eq!(v2_loaded.right_to_middle(), and_gate.right_to_middle());
    assert_eq!(v2_loaded.middle(), and_gate.middle());
}

// ---------------------------------------------------------------------------
// 4. Three-input NAND gate (wider fan-in)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn three_input_nand_gate() {
    let db = setup_both().await;
    let v2 = HyperedgeStore::new(&db);

    // NAND3: [A, B, C] → [Out]
    let nand3: Cospan<i32> = Cospan::new(vec![0, 1, 2], vec![3], vec![0, 1, 2, 3]);
    let names = ["A", "B", "C", "NAND_Out"];

    let hub_id = v2
        .decompose_cospan(
            &nand3,
            "NAND",
            serde_json::json!({"inputs": 3, "delay_ns": 3}),
            gate_name(&names),
        )
        .await
        .unwrap();

    let hub = v2.get_hub(&hub_id).await.unwrap();
    assert_eq!(hub.source_count, 3);
    assert_eq!(hub.target_count, 1);

    // Roundtrip
    let reconstructed: Cospan<i32> = v2.reconstruct_cospan(&hub_id).await.unwrap();
    assert_eq!(reconstructed.left_to_middle(), &[0, 1, 2]);
    assert_eq!(reconstructed.right_to_middle(), &[3]);
    assert_eq!(reconstructed.middle().len(), 4);
}

// ---------------------------------------------------------------------------
// 5. Buffer / identity gate: single pass-through
// ---------------------------------------------------------------------------

#[tokio::test]
async fn buffer_gate_identity() {
    let db = setup_both().await;
    let v2 = HyperedgeStore::new(&db);

    // Buffer: [Signal] → [Signal] (identity — same middle node)
    let buffer: Cospan<i32> = Cospan::new(vec![0], vec![0], vec![0]);
    let names = ["Signal"];

    let hub_id = v2
        .decompose_cospan(
            &buffer,
            "BUFFER",
            serde_json::json!({"delay_ns": 0}),
            gate_name(&names),
        )
        .await
        .unwrap();

    let hub = v2.get_hub(&hub_id).await.unwrap();
    assert_eq!(hub.source_count, 1);
    assert_eq!(hub.target_count, 1);

    // Single middle node, both source and target point to it
    let sources = v2.sources(&hub_id).await.unwrap();
    let targets = v2.targets(&hub_id).await.unwrap();
    assert_eq!(sources[0].name, "Signal");
    assert_eq!(targets[0].name, "Signal");

    // Roundtrip preserves identity structure
    let reconstructed: Cospan<i32> = v2.reconstruct_cospan(&hub_id).await.unwrap();
    assert_eq!(reconstructed.left_to_middle(), &[0]);
    assert_eq!(reconstructed.right_to_middle(), &[0]);
    assert_eq!(reconstructed.middle(), &[0]);
}
