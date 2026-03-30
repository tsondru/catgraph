//! Use Case: Blitzy-like Code Graph
//!
//! Models a code analysis graph with functions, modules, and call/containment
//! relationships. Demonstrates pairwise `NodeStore` + `EdgeStore` (no hyperedges),
//! edge properties, multi-hop traversal, and inbound queries.

use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

use catgraph_surreal::edge_store::EdgeStore;
use catgraph_surreal::init_schema_v2;
use catgraph_surreal::node_store::NodeStore;
use catgraph_surreal::query::QueryHelper;

async fn setup() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    init_schema_v2(&db).await.unwrap();
    db
}

// ---------------------------------------------------------------------------
// 1. Build a code graph and verify structure
// ---------------------------------------------------------------------------

#[tokio::test]
async fn code_graph_functions_and_calls() {
    let db = setup().await;
    let nodes = NodeStore::new(&db);
    let edges = EdgeStore::new(&db);

    // Create function nodes with source location properties
    let parse = nodes
        .create(
            "parse",
            "function",
            vec!["public".into()],
            serde_json::json!({"file": "main.rs", "line": 10}),
        )
        .await
        .unwrap();
    let validate = nodes
        .create(
            "validate",
            "function",
            vec!["public".into()],
            serde_json::json!({"file": "main.rs", "line": 50}),
        )
        .await
        .unwrap();
    let execute = nodes
        .create(
            "execute",
            "function",
            vec!["public".into(), "async".into()],
            serde_json::json!({"file": "main.rs", "line": 100}),
        )
        .await
        .unwrap();

    // Create call edges with call_count property
    edges
        .relate(
            &parse,
            &validate,
            "calls",
            None,
            serde_json::json!({"call_count": 42}),
        )
        .await
        .unwrap();
    edges
        .relate(
            &validate,
            &execute,
            "calls",
            None,
            serde_json::json!({"call_count": 7}),
        )
        .await
        .unwrap();

    // Verify outbound: parse calls validate
    let called_by_parse = edges.traverse_outbound(&parse, "calls").await.unwrap();
    assert_eq!(called_by_parse.len(), 1);
    assert_eq!(called_by_parse[0].name, "validate");

    // Verify inbound: who calls execute?
    let callers_of_execute = edges.traverse_inbound(&execute, "calls").await.unwrap();
    assert_eq!(callers_of_execute.len(), 1);
    assert_eq!(callers_of_execute[0].name, "validate");

    // Verify edge properties
    let between = edges.edges_between(&parse, &validate).await.unwrap();
    assert_eq!(between.len(), 1);
    assert_eq!(between[0].properties["call_count"], 42);
}

// ---------------------------------------------------------------------------
// 2. Module containment + mixed edge kinds
// ---------------------------------------------------------------------------

#[tokio::test]
async fn code_graph_modules_and_containment() {
    let db = setup().await;
    let nodes = NodeStore::new(&db);
    let edges = EdgeStore::new(&db);

    let core_mod = nodes
        .create("core", "module", vec![], serde_json::json!({"path": "src/core/"}))
        .await
        .unwrap();
    let parse = nodes
        .create("parse", "function", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let validate = nodes
        .create("validate", "function", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let helper = nodes
        .create("helper", "function", vec!["private".into()], serde_json::json!({}))
        .await
        .unwrap();

    // Containment edges
    edges
        .relate(&core_mod, &parse, "contains", None, serde_json::json!({}))
        .await
        .unwrap();
    edges
        .relate(&core_mod, &validate, "contains", None, serde_json::json!({}))
        .await
        .unwrap();
    edges
        .relate(&core_mod, &helper, "contains", None, serde_json::json!({}))
        .await
        .unwrap();

    // Call edges (different kind)
    edges
        .relate(&parse, &helper, "calls", None, serde_json::json!({}))
        .await
        .unwrap();

    // Module contains 3 functions
    let contained = edges.traverse_outbound(&core_mod, "contains").await.unwrap();
    assert_eq!(contained.len(), 3);

    // "contains" traversal from core doesn't include "calls" edges
    let calls_from_core = edges.traverse_outbound(&core_mod, "calls").await.unwrap();
    assert!(calls_from_core.is_empty());

    // parse calls helper (via "calls" kind)
    let called_by_parse = edges.traverse_outbound(&parse, "calls").await.unwrap();
    assert_eq!(called_by_parse.len(), 1);
    assert_eq!(called_by_parse[0].name, "helper");
}

// ---------------------------------------------------------------------------
// 3. Multi-hop traversal via QueryHelper
// ---------------------------------------------------------------------------

#[tokio::test]
async fn code_graph_multi_hop_traversal() {
    let db = setup().await;
    let nodes = NodeStore::new(&db);
    let edges = EdgeStore::new(&db);
    let query = QueryHelper::new(&db);

    // Chain: parse -> validate -> execute -> cleanup
    let parse = nodes
        .create("parse", "function", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let validate = nodes
        .create("validate", "function", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let execute = nodes
        .create("execute", "function", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let cleanup = nodes
        .create("cleanup", "function", vec![], serde_json::json!({}))
        .await
        .unwrap();

    edges
        .relate(&parse, &validate, "calls", None, serde_json::json!({}))
        .await
        .unwrap();
    edges
        .relate(&validate, &execute, "calls", None, serde_json::json!({}))
        .await
        .unwrap();
    edges
        .relate(&execute, &cleanup, "calls", None, serde_json::json!({}))
        .await
        .unwrap();

    // Depth 1: only validate
    let depth1 = query.reachable(&parse, "calls", 1).await.unwrap();
    assert_eq!(depth1.len(), 1);
    assert_eq!(depth1[0].name, "validate");

    // Depth 2: validate + execute
    let depth2 = query.reachable(&parse, "calls", 2).await.unwrap();
    assert_eq!(depth2.len(), 2);
    let names: Vec<&str> = depth2.iter().map(|n| n.name.as_str()).collect();
    assert!(names.contains(&"validate"));
    assert!(names.contains(&"execute"));

    // Depth 3: all three
    let depth3 = query.reachable(&parse, "calls", 3).await.unwrap();
    assert_eq!(depth3.len(), 3);
    let names: Vec<&str> = depth3.iter().map(|n| n.name.as_str()).collect();
    assert!(names.contains(&"validate"));
    assert!(names.contains(&"execute"));
    assert!(names.contains(&"cleanup"));
}

// ---------------------------------------------------------------------------
// 4. Find by kind — filtering node types
// ---------------------------------------------------------------------------

#[tokio::test]
async fn code_graph_find_by_kind() {
    let db = setup().await;
    let nodes = NodeStore::new(&db);

    nodes
        .create("parse", "function", vec![], serde_json::json!({}))
        .await
        .unwrap();
    nodes
        .create("validate", "function", vec![], serde_json::json!({}))
        .await
        .unwrap();
    nodes
        .create("execute", "function", vec![], serde_json::json!({}))
        .await
        .unwrap();
    nodes
        .create("core", "module", vec![], serde_json::json!({}))
        .await
        .unwrap();
    nodes
        .create("utils", "module", vec![], serde_json::json!({}))
        .await
        .unwrap();

    let functions = nodes.find_by_kind("function").await.unwrap();
    assert_eq!(functions.len(), 3);

    let modules = nodes.find_by_kind("module").await.unwrap();
    assert_eq!(modules.len(), 2);

    let structs = nodes.find_by_kind("struct").await.unwrap();
    assert!(structs.is_empty());
}

// ---------------------------------------------------------------------------
// 5. Diamond call graph — fan-out and fan-in
// ---------------------------------------------------------------------------

#[tokio::test]
async fn code_graph_diamond_pattern() {
    let db = setup().await;
    let nodes = NodeStore::new(&db);
    let edges = EdgeStore::new(&db);
    let query = QueryHelper::new(&db);

    // Diamond: entry -> [left, right] -> merge
    let entry = nodes
        .create("entry", "function", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let left = nodes
        .create("left_branch", "function", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let right = nodes
        .create("right_branch", "function", vec![], serde_json::json!({}))
        .await
        .unwrap();
    let merge = nodes
        .create("merge", "function", vec![], serde_json::json!({}))
        .await
        .unwrap();

    edges
        .relate(&entry, &left, "calls", None, serde_json::json!({}))
        .await
        .unwrap();
    edges
        .relate(&entry, &right, "calls", None, serde_json::json!({}))
        .await
        .unwrap();
    edges
        .relate(&left, &merge, "calls", None, serde_json::json!({}))
        .await
        .unwrap();
    edges
        .relate(&right, &merge, "calls", None, serde_json::json!({}))
        .await
        .unwrap();

    // entry fans out to 2
    let from_entry = edges.traverse_outbound(&entry, "calls").await.unwrap();
    assert_eq!(from_entry.len(), 2);

    // merge has 2 callers
    let to_merge = edges.traverse_inbound(&merge, "calls").await.unwrap();
    assert_eq!(to_merge.len(), 2);

    // Reachable from entry at depth 2: left, right, merge (3 nodes)
    let reachable = query.reachable(&entry, "calls", 2).await.unwrap();
    assert_eq!(reachable.len(), 3);

    // Inbound neighbors of merge
    let callers = query.inbound_neighbors(&merge, "calls").await.unwrap();
    assert_eq!(callers.len(), 2);
    let names: Vec<&str> = callers.iter().map(|n| n.name.as_str()).collect();
    assert!(names.contains(&"left_branch"));
    assert!(names.contains(&"right_branch"));
}
