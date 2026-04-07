//! Graph traversal: outbound/inbound neighbors, reachability, shortest path.
//!
//! Builds a small directed graph using NodeStore + EdgeStore, then demonstrates
//! QueryHelper's BFS-based traversal methods. Models a flow network with
//! two paths from A to D.
//!
//! Run with: `cargo run -p catgraph-surreal --example graph_traversal`

use catgraph_surreal::edge_store::EdgeStore;
use catgraph_surreal::init_schema_v2;
use catgraph_surreal::node_store::NodeStore;
use catgraph_surreal::query::QueryHelper;
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
    let qh = QueryHelper::new(&db);

    // --- Build a directed graph ---
    //
    //   A ---> B ---> C ---> D
    //   |                    ^
    //   +-------> E ---------+
    //
    let a = ns.create("A", "vertex", vec![], serde_json::json!({})).await?;
    let b = ns.create("B", "vertex", vec![], serde_json::json!({})).await?;
    let c = ns.create("C", "vertex", vec![], serde_json::json!({})).await?;
    let d = ns.create("D", "vertex", vec![], serde_json::json!({})).await?;
    let e = ns.create("E", "vertex", vec![], serde_json::json!({})).await?;

    es.relate(&a, &b, "flow", None, serde_json::json!({})).await?;
    es.relate(&b, &c, "flow", None, serde_json::json!({})).await?;
    es.relate(&c, &d, "flow", None, serde_json::json!({})).await?;
    es.relate(&a, &e, "flow", None, serde_json::json!({})).await?;
    es.relate(&e, &d, "flow", None, serde_json::json!({})).await?;

    println!("Built graph: 5 nodes, 5 edges");
    println!("  A -> B -> C -> D");
    println!("  A -> E -> D");

    // --- Outbound neighbors of A ---
    let out_a = qh.outbound_neighbors(&a, "flow").await?;
    let names: Vec<&str> = out_a.iter().map(|n| n.name.as_str()).collect();
    println!("\nOutbound neighbors of A: {names:?}");

    // --- Inbound neighbors of D ---
    let in_d = qh.inbound_neighbors(&d, "flow").await?;
    let names: Vec<&str> = in_d.iter().map(|n| n.name.as_str()).collect();
    println!("Inbound neighbors of D: {names:?}");

    // --- Reachable from A within 5 hops ---
    let reachable = qh.reachable(&a, "flow", 5).await?;
    let names: Vec<&str> = reachable.iter().map(|n| n.name.as_str()).collect();
    println!("\nAll nodes reachable from A (depth 5): {names:?}");
    println!("  Count: {}", reachable.len());

    // --- Shortest path from A to D ---
    let path = qh.shortest_path(&a, &d, "flow", 5).await?;
    match &path {
        Some(p) => {
            let names: Vec<&str> = p.iter().map(|n| n.name.as_str()).collect();
            println!("\nShortest path A -> D: {names:?}");
            println!("  Hops: {}", p.len() - 1);
        }
        None => println!("\nNo path found from A to D"),
    }

    // --- Shortest path from A to C (longer path) ---
    let path_ac = qh.shortest_path(&a, &c, "flow", 5).await?;
    if let Some(p) = &path_ac {
        let names: Vec<&str> = p.iter().map(|n| n.name.as_str()).collect();
        println!("Shortest path A -> C: {names:?}");
        println!("  Hops: {}", p.len() - 1);
    }

    // --- Collect reachable (alias for reachable) ---
    let collected = qh.collect_reachable(&e, "flow", 3).await?;
    let names: Vec<&str> = collected.iter().map(|n| n.name.as_str()).collect();
    println!("\nAll reachable from E (depth 3): {names:?}");

    println!("\nGraph traversal complete");

    Ok(())
}
