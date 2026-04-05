//! Structural fingerprint computation and HNSW similarity search.
//!
//! Builds a small graph, computes structural fingerprints for each node,
//! stores them as embeddings, then searches for structurally similar nodes.
//!
//! Run with: `cargo run -p catgraph-surreal --example fingerprint_search`

use catgraph_surreal::edge_store::EdgeStore;
use catgraph_surreal::fingerprint::FingerprintEngine;
use catgraph_surreal::init_schema_v2;
use catgraph_surreal::node_store::NodeStore;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Setup ---
    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("demo").use_db("demo").await?;
    init_schema_v2(&db).await?;

    let ns = NodeStore::new(&db);
    let es = EdgeStore::new(&db);
    let fe = FingerprintEngine::new(&db, 8); // 8-dimensional fingerprints
    fe.init_index().await?;

    // --- Build a graph ---
    // Star topology: hub connected to 4 leaves
    let hub = ns.create("hub", "router", vec![], serde_json::json!({})).await?;
    let leaf_a = ns.create("leaf_a", "server", vec![], serde_json::json!({})).await?;
    let leaf_b = ns.create("leaf_b", "server", vec![], serde_json::json!({})).await?;
    let leaf_c = ns.create("leaf_c", "server", vec![], serde_json::json!({})).await?;
    let leaf_d = ns.create("leaf_d", "server", vec![], serde_json::json!({})).await?;

    es.relate(&hub, &leaf_a, "link", None, serde_json::json!({})).await?;
    es.relate(&hub, &leaf_b, "link", None, serde_json::json!({})).await?;
    es.relate(&hub, &leaf_c, "link", None, serde_json::json!({})).await?;
    es.relate(&hub, &leaf_d, "link", None, serde_json::json!({})).await?;

    // Chain: x -> y -> z
    let x = ns.create("chain_start", "endpoint", vec![], serde_json::json!({})).await?;
    let y = ns.create("chain_mid", "relay", vec![], serde_json::json!({})).await?;
    let z = ns.create("chain_end", "endpoint", vec![], serde_json::json!({})).await?;

    es.relate(&x, &y, "link", None, serde_json::json!({})).await?;
    es.relate(&y, &z, "link", None, serde_json::json!({})).await?;

    println!("Built graph: 7 nodes, 6 edges");

    // --- Compute and store fingerprints ---
    for (name, id) in [
        ("hub", &hub), ("leaf_a", &leaf_a), ("leaf_b", &leaf_b),
        ("leaf_c", &leaf_c), ("leaf_d", &leaf_d),
        ("chain_start", &x), ("chain_mid", &y), ("chain_end", &z),
    ] {
        let fp = fe.index_node(id).await?;
        println!("  {name:>12}: [{:.0}, {:.0}, {:.0}, {:.0}, ...]",
            fp[0], fp[1], fp[2], fp[3]);
    }

    // --- Search for nodes similar to the hub ---
    println!("\nSearching for nodes similar to 'hub' (out-degree=4):");
    let hub_fp = fe.compute_fingerprint(&hub).await?;
    let results = fe.search_similar(&hub_fp, 3, 50).await?;
    for (node, distance) in &results {
        println!("  {} (distance={:.4})", node.name, distance);
    }

    // --- Search for nodes similar to a leaf ---
    println!("\nSearching for nodes similar to 'leaf_a' (out-degree=0):");
    let leaf_fp = fe.compute_fingerprint(&leaf_a).await?;
    let results = fe.search_similar(&leaf_fp, 3, 50).await?;
    for (node, distance) in &results {
        println!("  {} (distance={:.4})", node.name, distance);
    }

    Ok(())
}
