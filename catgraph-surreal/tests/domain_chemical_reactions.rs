//! Use Case: Chemical Reactions
//!
//! Models chemical reactions as hyperedges: reactants (sources) flow through
//! a reaction hub to produce products (targets). Demonstrates `Cospan` decomposition
//! via `HyperedgeStore`, hub properties, source/target queries, and roundtrip
//! reconstruction.

use catgraph::cospan::Cospan;
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

/// Map `i32` middle indices to chemical names.
fn chemical_name<'a>(names: &'a [&'a str]) -> impl Fn(&i32) -> String + 'a {
    move |i: &i32| names[usize::try_from(*i).expect("non-negative index")].to_string()
}

// ---------------------------------------------------------------------------
// 1. Combustion: H2 + O2 → H2O
// ---------------------------------------------------------------------------

#[tokio::test]
async fn combustion_decompose_and_query() {
    let db = setup().await;
    let store = HyperedgeStore::new(&db);

    // Cospan: left=[H2, O2] → middle=[H2, O2, H2O] ← right=[H2O]
    // left_map:  [0, 1]  (H2→middle[0], O2→middle[1])
    // right_map: [2]     (H2O→middle[2])
    let cospan: Cospan<i32> = Cospan::new(vec![0, 1], vec![2], vec![0, 1, 2]);
    let names = ["H2", "O2", "H2O"];

    let hub_id = store
        .decompose_cospan(
            &cospan,
            "combustion",
            serde_json::json!({"energy": "exothermic", "temperature_k": 500}),
            chemical_name(&names),
        )
        .await
        .unwrap();

    // Query sources (reactants)
    let sources = store.sources(&hub_id).await.unwrap();
    assert_eq!(sources.len(), 2);
    let source_names: Vec<&str> = sources.iter().map(|n| n.name.as_str()).collect();
    assert!(source_names.contains(&"H2"));
    assert!(source_names.contains(&"O2"));

    // Query targets (products)
    let targets = store.targets(&hub_id).await.unwrap();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].name, "H2O");

    // Verify hub properties
    let hub = store.get_hub(&hub_id).await.unwrap();
    assert_eq!(hub.kind, "combustion");
    assert_eq!(hub.properties["energy"], "exothermic");
    assert_eq!(hub.properties["temperature_k"], 500);
    assert_eq!(hub.source_count, 2);
    assert_eq!(hub.target_count, 1);
}

// ---------------------------------------------------------------------------
// 2. Roundtrip reconstruction
// ---------------------------------------------------------------------------

#[tokio::test]
async fn combustion_reconstruct_roundtrip() {
    let db = setup().await;
    let store = HyperedgeStore::new(&db);

    let cospan: Cospan<i32> = Cospan::new(vec![0, 1], vec![2], vec![0, 1, 2]);
    let names = ["H2", "O2", "H2O"];

    let hub_id = store
        .decompose_cospan(&cospan, "combustion", serde_json::json!({}), chemical_name(&names))
        .await
        .unwrap();

    let reconstructed: Cospan<i32> = store.reconstruct_cospan(&hub_id).await.unwrap();

    assert_eq!(reconstructed.left_to_middle(), cospan.left_to_middle());
    assert_eq!(reconstructed.right_to_middle(), cospan.right_to_middle());
    assert_eq!(reconstructed.middle(), cospan.middle());
}

// ---------------------------------------------------------------------------
// 3. Synthesis with shared intermediate: A + B → C, C + D → E
// ---------------------------------------------------------------------------

#[tokio::test]
async fn two_step_synthesis() {
    let db = setup().await;
    let store = HyperedgeStore::new(&db);

    // Step 1: A + B → C
    // middle = [A=0, B=1, C=2], left_map=[0,1], right_map=[2]
    let step1: Cospan<i32> = Cospan::new(vec![0, 1], vec![2], vec![0, 1, 2]);
    let step1_names = ["reagent_A", "reagent_B", "intermediate_C"];

    let hub1 = store
        .decompose_cospan(
            &step1,
            "synthesis_step_1",
            serde_json::json!({"catalyst": "Pd", "yield_pct": 85}),
            chemical_name(&step1_names),
        )
        .await
        .unwrap();

    // Step 2: C + D → E
    // middle = [C=0, D=1, E=2], left_map=[0,1], right_map=[2]
    let step2: Cospan<i32> = Cospan::new(vec![0, 1], vec![2], vec![0, 1, 2]);
    let step2_names = ["intermediate_C", "reagent_D", "product_E"];

    let hub2 = store
        .decompose_cospan(
            &step2,
            "synthesis_step_2",
            serde_json::json!({"catalyst": "Pt", "yield_pct": 92}),
            chemical_name(&step2_names),
        )
        .await
        .unwrap();

    // Step 1 produces intermediate_C
    let products1 = store.targets(&hub1).await.unwrap();
    assert_eq!(products1.len(), 1);
    assert_eq!(products1[0].name, "intermediate_C");

    // Step 2 consumes intermediate_C (as a source)
    let reactants2 = store.sources(&hub2).await.unwrap();
    assert_eq!(reactants2.len(), 2);
    let reactant_names: Vec<&str> = reactants2.iter().map(|n| n.name.as_str()).collect();
    assert!(reactant_names.contains(&"intermediate_C"));
    assert!(reactant_names.contains(&"reagent_D"));

    // Final product
    let products2 = store.targets(&hub2).await.unwrap();
    assert_eq!(products2.len(), 1);
    assert_eq!(products2[0].name, "product_E");

    // Both hubs have independent metadata
    let h1 = store.get_hub(&hub1).await.unwrap();
    let h2 = store.get_hub(&hub2).await.unwrap();
    assert_eq!(h1.properties["catalyst"], "Pd");
    assert_eq!(h2.properties["catalyst"], "Pt");
}

// ---------------------------------------------------------------------------
// 4. Reversible reaction: same species on both sides
// ---------------------------------------------------------------------------

#[tokio::test]
async fn reversible_reaction_shared_species() {
    let db = setup().await;
    let store = HyperedgeStore::new(&db);

    // Equilibrium: N2 + 3H2 ⇌ 2NH3
    // Forward: sources=[N2, H2, H2, H2] → targets=[NH3, NH3]
    // middle = [N2=0, H2=1, NH3=2]
    // left_map:  [0, 1, 1, 1]  (N2 once, H2 three times — same middle index)
    // right_map: [2, 2]        (NH3 twice)
    let forward: Cospan<i32> = Cospan::new(vec![0, 1, 1, 1], vec![2, 2], vec![0, 1, 2]);
    let names = ["N2", "H2", "NH3"];

    let hub_id = store
        .decompose_cospan(
            &forward,
            "haber_process",
            serde_json::json!({"reversible": true, "pressure_atm": 200}),
            chemical_name(&names),
        )
        .await
        .unwrap();

    let hub = store.get_hub(&hub_id).await.unwrap();
    assert_eq!(hub.source_count, 4); // 4 source participations (stoichiometric)
    assert_eq!(hub.target_count, 2); // 2 target participations

    // Roundtrip preserves stoichiometry
    let reconstructed: Cospan<i32> = store.reconstruct_cospan(&hub_id).await.unwrap();
    assert_eq!(reconstructed.left_to_middle(), &[0, 1, 1, 1]);
    assert_eq!(reconstructed.right_to_middle(), &[2, 2]);
}

// ---------------------------------------------------------------------------
// 5. Delete reaction hub and verify cleanup
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_reaction_hub() {
    let db = setup().await;
    let store = HyperedgeStore::new(&db);

    let cospan: Cospan<i32> = Cospan::new(vec![0], vec![1], vec![0, 1]);
    let names = ["reactant", "product"];

    let hub_id = store
        .decompose_cospan(&cospan, "simple", serde_json::json!({}), chemical_name(&names))
        .await
        .unwrap();

    // Hub exists
    store.get_hub(&hub_id).await.unwrap();

    // Delete
    store.delete_hub(&hub_id).await.unwrap();

    // Hub gone
    assert!(store.get_hub(&hub_id).await.is_err());
}
