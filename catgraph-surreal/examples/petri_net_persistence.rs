//! Petri net persistence: save topology, fire transitions, persist markings.
//!
//! Demonstrates the full lifecycle: create a chemical reaction net,
//! save it to SurrealDB, fire a transition, persist the resulting marking,
//! then load everything back.
//!
//! Run with: `cargo run -p catgraph-surreal --example petri_net_persistence`

use catgraph::petri_net::{Marking, PetriNet, Transition};
use catgraph_surreal::petri_net_store::PetriNetStore;
use catgraph_surreal::init_schema_v2;
use rust_decimal::Decimal;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

fn d(n: i64) -> Decimal {
    Decimal::from(n)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Setup ---
    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("demo").use_db("demo").await?;
    init_schema_v2(&db).await?;
    let store = PetriNetStore::new(&db);

    // --- Build a combustion net: 2H2 + O2 → 2H2O ---
    let net: PetriNet<char> = PetriNet::new(
        vec!['H', 'O', 'W'], // H=hydrogen, O=oxygen, W=water
        vec![Transition::new(
            vec![(0, d(2)), (1, d(1))], // consume 2H, 1O
            vec![(2, d(2))],            // produce 2W
        )],
    );
    println!("Built combustion net: {} places, {} transitions",
        net.place_count(), net.transition_count());

    // --- Save to SurrealDB ---
    let net_id = store.save(&net, "combustion").await?;
    println!("Saved net: {net_id:?}");

    // --- Save initial marking ---
    let m0 = Marking::from_vec(vec![(0, d(4)), (1, d(2))]);
    let m0_id = store.save_marking(&net_id, &m0, "initial").await?;
    println!("Saved initial marking: {m0_id:?}");
    println!("  H={}, O={}, W={}", m0.get(0), m0.get(1), m0.get(2));

    // --- Fire transition 0 ---
    let m1 = net.fire(0, &m0)?;
    let m1_id = store.save_marking(&net_id, &m1, "after_fire_1").await?;
    println!("After first firing: {m1_id:?}");
    println!("  H={}, O={}, W={}", m1.get(0), m1.get(1), m1.get(2));

    // --- Fire again ---
    let m2 = net.fire(0, &m1)?;
    let m2_id = store.save_marking(&net_id, &m2, "after_fire_2").await?;
    println!("After second firing: {m2_id:?}");
    println!("  H={}, O={}, W={}", m2.get(0), m2.get(1), m2.get(2));

    // --- Load back ---
    let loaded: PetriNet<char> = store.load(&net_id).await?;
    println!("\nLoaded net: {} places, {} transitions",
        loaded.place_count(), loaded.transition_count());
    println!("  arc_weight_pre(H, t0) = {}", loaded.arc_weight_pre(0, 0));
    println!("  arc_weight_post(W, t0) = {}", loaded.arc_weight_post(2, 0));

    let loaded_m2 = store.load_marking(&m2_id).await?;
    println!("Loaded final marking: H={}, O={}, W={}",
        loaded_m2.get(0), loaded_m2.get(1), loaded_m2.get(2));

    // --- List all nets ---
    let nets = store.list().await?;
    println!("\n{} net(s) in database", nets.len());

    // --- Cleanup ---
    store.delete(&net_id).await?;
    println!("Deleted net and all associated records");

    Ok(())
}
