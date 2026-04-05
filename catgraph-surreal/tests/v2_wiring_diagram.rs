//! Integration tests for WiringDiagramStore (V2 persistence).

use catgraph::named_cospan::NamedCospan;
use catgraph::wiring_diagram::{Dir, WiringDiagram};
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

use catgraph_surreal::wiring_store::WiringDiagramStore;
use catgraph_surreal::init_schema_v2;

async fn setup() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    init_schema_v2(&db).await.unwrap();
    db
}

/// Build a simple wiring diagram:
/// - Lambda = char, InterCircle = i32, IntraCircle = usize
/// - 3 middle nodes: 'a', 'b', 'c'
/// - Left (inner) ports: (In, 0, 0)→mid0, (Out, 0, 1)→mid1
/// - Right (outer) ports: (Out, 0)→mid0, (In, 1)→mid2
fn simple_diagram() -> WiringDiagram<char, i32, usize> {
    WiringDiagram::new(NamedCospan::new(
        vec![0, 1],       // left → middle
        vec![0, 2],       // right → middle
        vec!['a', 'b', 'c'],
        vec![(Dir::In, 0, 0), (Dir::Out, 0, 1)],
        vec![(Dir::Out, 0), (Dir::In, 1)],
    ))
}

/// Leaf diagram with no inner circles.
fn leaf_diagram() -> WiringDiagram<char, i32, usize> {
    WiringDiagram::new(NamedCospan::new(
        vec![],
        vec![0, 1],
        vec!['x', 'y'],
        vec![],
        vec![(Dir::In, 0), (Dir::Out, 1)],
    ))
}

// ---------------------------------------------------------------------------
// Save and retrieve hub
// ---------------------------------------------------------------------------

#[tokio::test]
async fn save_and_get_hub() {
    let db = setup().await;
    let store = WiringDiagramStore::new(&db);
    let diagram = simple_diagram();

    let hub_id = store.save(&diagram, "test_diagram").await.unwrap();
    let hub = store.get_hub(&hub_id).await.unwrap();

    assert_eq!(hub.kind, "wiring_diagram");
    assert_eq!(hub.source_count, 2); // left boundary size
    assert_eq!(hub.target_count, 2); // right boundary size

    let name = hub.properties.get("diagram_name").unwrap().as_str().unwrap();
    assert_eq!(name, "test_diagram");
}

// ---------------------------------------------------------------------------
// Save and load roundtrip
// ---------------------------------------------------------------------------

#[tokio::test]
async fn save_load_roundtrip() {
    let db = setup().await;
    let store = WiringDiagramStore::new(&db);
    let diagram = simple_diagram();

    let hub_id = store.save(&diagram, "roundtrip").await.unwrap();
    let loaded: WiringDiagram<char, i32, usize> = store.load(&hub_id).await.unwrap();

    let orig = diagram.inner();
    let rest = loaded.inner();

    // Middle sets match
    assert_eq!(orig.cospan().middle(), rest.cospan().middle());

    // Left/right maps match
    assert_eq!(orig.cospan().left_to_middle(), rest.cospan().left_to_middle());
    assert_eq!(orig.cospan().right_to_middle(), rest.cospan().right_to_middle());

    // Port names match
    assert_eq!(orig.left_names(), rest.left_names());
    assert_eq!(orig.right_names(), rest.right_names());
}

// ---------------------------------------------------------------------------
// Leaf diagram (empty left boundary) roundtrip
// ---------------------------------------------------------------------------

#[tokio::test]
async fn leaf_diagram_roundtrip() {
    let db = setup().await;
    let store = WiringDiagramStore::new(&db);
    let diagram = leaf_diagram();

    let hub_id = store.save(&diagram, "leaf").await.unwrap();
    let hub = store.get_hub(&hub_id).await.unwrap();
    assert_eq!(hub.source_count, 0);
    assert_eq!(hub.target_count, 2);

    let loaded: WiringDiagram<char, i32, usize> = store.load(&hub_id).await.unwrap();
    let orig = diagram.inner();
    let rest = loaded.inner();

    assert_eq!(orig.cospan().middle(), rest.cospan().middle());
    assert!(rest.left_names().is_empty());
    assert_eq!(orig.right_names(), rest.right_names());
}

// ---------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_wiring_diagram() {
    let db = setup().await;
    let store = WiringDiagramStore::new(&db);
    let diagram = simple_diagram();

    let hub_id = store.save(&diagram, "to_delete").await.unwrap();
    store.get_hub(&hub_id).await.unwrap(); // exists

    store.delete(&hub_id).await.unwrap();

    let err = store.get_hub(&hub_id).await;
    assert!(err.is_err());
}

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_wiring_diagrams() {
    let db = setup().await;
    let store = WiringDiagramStore::new(&db);

    // Initially empty
    let initial = store.list().await.unwrap();
    assert!(initial.is_empty());

    // Save two diagrams
    store.save(&simple_diagram(), "first").await.unwrap();
    store.save(&leaf_diagram(), "second").await.unwrap();

    let listed = store.list().await.unwrap();
    assert_eq!(listed.len(), 2);

    let names: Vec<&str> = listed
        .iter()
        .filter_map(|h| h.properties.get("diagram_name").and_then(|v| v.as_str()))
        .collect();
    assert!(names.contains(&"first"));
    assert!(names.contains(&"second"));
}

// ---------------------------------------------------------------------------
// List filters by kind
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_excludes_non_wiring_hubs() {
    let db = setup().await;
    let store = WiringDiagramStore::new(&db);

    // Save a wiring diagram
    store.save(&simple_diagram(), "wd").await.unwrap();

    // Save a non-wiring hub via HyperedgeStore directly
    let hs = catgraph_surreal::hyperedge_store::HyperedgeStore::new(&db);
    let cospan = catgraph::cospan::Cospan::new(vec![0], vec![0], vec!['z']);
    hs.decompose_cospan(&cospan, "other_kind", serde_json::json!({}), |c| c.to_string())
        .await
        .unwrap();

    // list() should only return the wiring diagram hub
    let listed = store.list().await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].kind, "wiring_diagram");
}

// ---------------------------------------------------------------------------
// Port metadata preserved in hub properties
// ---------------------------------------------------------------------------

#[tokio::test]
async fn port_metadata_in_properties() {
    let db = setup().await;
    let store = WiringDiagramStore::new(&db);
    let diagram = simple_diagram();

    let hub_id = store.save(&diagram, "meta").await.unwrap();
    let hub = store.get_hub(&hub_id).await.unwrap();

    // Left port names are stored
    let left_ports = hub.properties.get("left_port_names").unwrap();
    assert!(left_ports.is_array());
    let left_arr = left_ports.as_array().unwrap();
    assert_eq!(left_arr.len(), 2);

    // First left port: (Dir::In, 0, 0)
    assert_eq!(left_arr[0]["dir"], "In");
    assert_eq!(left_arr[0]["inter"], 0);
    assert_eq!(left_arr[0]["intra"], 0);

    // Second left port: (Dir::Out, 0, 1)
    assert_eq!(left_arr[1]["dir"], "Out");
    assert_eq!(left_arr[1]["inter"], 0);
    assert_eq!(left_arr[1]["intra"], 1);

    // Right port names
    let right_ports = hub.properties.get("right_port_names").unwrap();
    let right_arr = right_ports.as_array().unwrap();
    assert_eq!(right_arr.len(), 2);
    assert_eq!(right_arr[0]["dir"], "Out");
    assert_eq!(right_arr[0]["intra"], 0);
    assert_eq!(right_arr[1]["dir"], "In");
    assert_eq!(right_arr[1]["intra"], 1);
}

// ---------------------------------------------------------------------------
// Load wrong hub kind
// ---------------------------------------------------------------------------

#[tokio::test]
async fn load_wrong_kind_errors() {
    let db = setup().await;
    let store = WiringDiagramStore::new(&db);

    // Create a non-wiring hub
    let hs = catgraph_surreal::hyperedge_store::HyperedgeStore::new(&db);
    let cospan = catgraph::cospan::Cospan::new(vec![0], vec![0], vec!['z']);
    let hub_id = hs
        .decompose_cospan(&cospan, "reaction", serde_json::json!({}), |c| c.to_string())
        .await
        .unwrap();

    // Trying to load it as a wiring diagram should fail
    let result: Result<WiringDiagram<char, i32, usize>, _> = store.load(&hub_id).await;
    match result {
        Err(e) => {
            let err_msg = format!("{e}");
            assert!(err_msg.contains("wiring_diagram"), "unexpected error: {err_msg}");
        }
        Ok(_) => panic!("expected error loading non-wiring hub as WiringDiagram"),
    }
}
