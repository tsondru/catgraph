//! V1 CospanStore roundtrip: save, load, list, and delete cospans.
//!
//! Demonstrates the simplest persistence layer — embedded arrays in the V1 schema.
//! Each cospan is stored as a single record with left/right maps and middle labels.
//!
//! Run with: `cargo run -p catgraph-surreal --example cospan_v1_roundtrip`

use catgraph::category::HasIdentity;
use catgraph::cospan::Cospan;
use catgraph_surreal::cospan_store::CospanStore;
use catgraph_surreal::init_schema;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Setup ---
    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("demo").use_db("demo").await?;
    init_schema(&db).await?;
    let store = CospanStore::new(&db);

    // --- Build a cospan: [a, b] -> [c] with shared middle vertex ---
    // Middle set has 3 elements: 'a', 'b', 'c'
    // Source 0 -> middle 0 ('a'), source 1 -> middle 1 ('b')
    // Target 0 -> middle 2 ('c')
    let cospan = Cospan::new(
        vec![0, 1],          // left_map
        vec![2],             // right_map
        vec!['a', 'b', 'c'], // middle
    );
    println!("Built cospan:");
    println!("  left_map:  {:?}", cospan.left_to_middle());
    println!("  right_map: {:?}", cospan.right_to_middle());
    println!("  middle:    {:?}", cospan.middle());

    // --- Save ---
    let id = store.save(&cospan).await?;
    println!("\nSaved cospan: {id:?}");

    // --- Load and verify ---
    let loaded: Cospan<char> = store.load(&id).await?;
    println!("Loaded cospan:");
    println!("  left_map:  {:?}", loaded.left_to_middle());
    println!("  right_map: {:?}", loaded.right_to_middle());
    println!("  middle:    {:?}", loaded.middle());

    assert_eq!(cospan.left_to_middle(), loaded.left_to_middle());
    assert_eq!(cospan.right_to_middle(), loaded.right_to_middle());
    assert_eq!(cospan.middle(), loaded.middle());
    println!("  Roundtrip verified!");

    // --- List stored cospans ---
    let ids = store.list().await?;
    println!("\nStored cospans: {} record(s)", ids.len());

    // --- Build and save an identity cospan ---
    let identity = Cospan::identity(&vec!['x', 'y', 'z']);
    println!("\nBuilt identity cospan on ['x', 'y', 'z']:");
    println!("  is_left_identity:  {}", identity.is_left_identity());
    println!("  is_right_identity: {}", identity.is_right_identity());

    let id_id = store.save(&identity).await?;
    println!("Saved identity: {id_id:?}");

    let loaded_id: Cospan<char> = store.load(&id_id).await?;
    assert!(loaded_id.is_left_identity());
    assert!(loaded_id.is_right_identity());
    println!("  Identity flags preserved after roundtrip!");

    // --- List again ---
    let ids = store.list().await?;
    println!("\nStored cospans: {} record(s)", ids.len());

    // --- Delete first cospan ---
    store.delete(&id).await?;
    println!("Deleted first cospan");

    store.delete(&id_id).await?;
    println!("Deleted identity cospan");

    // --- Verify empty ---
    let ids = store.list().await?;
    println!("\nStored cospans after cleanup: {} record(s)", ids.len());
    assert!(ids.is_empty());
    println!("Store is empty");

    Ok(())
}
