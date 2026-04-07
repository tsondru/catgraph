//! Hyperedge hub-node roundtrip: decompose cospans and named cospans, then reconstruct.
//!
//! Demonstrates the most complex V2 store — `HyperedgeStore` — which reifies
//! n-ary hyperedges as hub nodes with positional source/target edges.
//! Shows Cospan and NamedCospan decompose/reconstruct workflows.
//!
//! Run with: `cargo run -p catgraph-surreal --example hyperedge_roundtrip`

use catgraph::cospan::Cospan;
use catgraph::named_cospan::NamedCospan;
use catgraph_surreal::hyperedge_store::HyperedgeStore;
use catgraph_surreal::init_schema_v2;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Setup ---
    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("demo").use_db("demo").await?;
    init_schema_v2(&db).await?;
    let store = HyperedgeStore::new(&db);

    // --- Build a Cospan representing a chemical reaction: [H, O] -> [W] ---
    // Middle set: H (hydrogen), O (oxygen), W (water)
    // Left (sources) map:  index 0 -> H (middle 0), index 1 -> O (middle 1)
    // Right (targets) map: index 0 -> W (middle 2)
    let cospan = Cospan::new(
        vec![0, 1],    // left_map: source 0 -> middle 0 (H), source 1 -> middle 1 (O)
        vec![2],       // right_map: target 0 -> middle 2 (W)
        vec!['H', 'O', 'W'],
    );
    println!("Built cospan: [H, O] -> [W]");
    println!("  left_map:  {:?}", cospan.left_to_middle());
    println!("  right_map: {:?}", cospan.right_to_middle());
    println!("  middle:    {:?}", cospan.middle());

    // --- Decompose into hub-node reification ---
    let hub_id = store
        .decompose_cospan(&cospan, "reaction", serde_json::json!({"name": "combustion"}), |c| {
            c.to_string()
        })
        .await?;
    println!("\nDecomposed to hub: {hub_id:?}");

    let hub = store.get_hub(&hub_id).await?;
    println!("  kind:         {}", hub.kind);
    println!("  source_count: {}", hub.source_count);
    println!("  target_count: {}", hub.target_count);

    // --- Query sources and targets ---
    let sources = store.sources(&hub_id).await?;
    println!("\nSources ({}):", sources.len());
    for s in &sources {
        println!("  {} (kind={})", s.name, s.kind);
    }

    let targets = store.targets(&hub_id).await?;
    println!("Targets ({}):", targets.len());
    for t in &targets {
        println!("  {} (kind={})", t.name, t.kind);
    }

    // --- Reconstruct and verify ---
    let reconstructed: Cospan<char> = store.reconstruct_cospan(&hub_id).await?;
    println!("\nReconstructed cospan:");
    println!("  left_map:  {:?}", reconstructed.left_to_middle());
    println!("  right_map: {:?}", reconstructed.right_to_middle());
    println!("  middle:    {:?}", reconstructed.middle());

    assert_eq!(cospan.left_to_middle(), reconstructed.left_to_middle());
    assert_eq!(cospan.right_to_middle(), reconstructed.right_to_middle());
    assert_eq!(cospan.middle(), reconstructed.middle());
    println!("  Roundtrip verified!");

    // --- Cleanup first hub ---
    store.delete_hub(&hub_id).await?;

    // --- Named Cospan with port labels ---
    println!("\n--- Named Cospan ---");
    let named = NamedCospan::new(
        vec![0, 1],    // same structural maps
        vec![2],
        vec!['H', 'O', 'W'],
        vec!["hydrogen".to_string(), "oxygen".to_string()],
        vec!["water".to_string()],
    );
    println!("Built named cospan with ports:");
    println!("  left ports:  {:?}", named.left_names());
    println!("  right ports: {:?}", named.right_names());

    let named_hub_id = store
        .decompose_named_cospan(
            &named,
            "reaction",
            serde_json::json!({"name": "combustion_named"}),
        )
        .await?;
    println!("Decomposed to hub: {named_hub_id:?}");

    let named_hub = store.get_hub(&named_hub_id).await?;
    println!("  left_port_names:  {:?}", named_hub.properties.get("left_port_names"));
    println!("  right_port_names: {:?}", named_hub.properties.get("right_port_names"));

    // --- Reconstruct named cospan and verify port names ---
    let reconstructed_named: NamedCospan<char, String, String> =
        store.reconstruct_named_cospan(&named_hub_id).await?;
    println!("\nReconstructed named cospan:");
    println!("  left ports:  {:?}", reconstructed_named.left_names());
    println!("  right ports: {:?}", reconstructed_named.right_names());
    println!("  middle:      {:?}", reconstructed_named.cospan().middle());

    assert_eq!(named.left_names(), reconstructed_named.left_names());
    assert_eq!(named.right_names(), reconstructed_named.right_names());
    assert_eq!(named.cospan().middle(), reconstructed_named.cospan().middle());
    println!("  Named roundtrip verified!");

    // --- Cleanup ---
    store.delete_hub(&named_hub_id).await?;
    println!("\nCleaned up all hub records");

    Ok(())
}
