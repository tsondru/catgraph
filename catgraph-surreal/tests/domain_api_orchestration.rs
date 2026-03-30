//! Use Case: API Orchestration
//!
//! Models service orchestration where multiple input services contribute to
//! a composite operation producing output artifacts. Demonstrates `Cospan`
//! decomposition with rich hub properties (timeout, MFA requirements) and
//! querying operation inputs/outputs.

use catgraph::cospan::Cospan;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

use catgraph_surreal::hyperedge_store::HyperedgeStore;
use catgraph_surreal::init_schema_v2;
use catgraph_surreal::node_store::NodeStore;

async fn setup() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    init_schema_v2(&db).await.unwrap();
    db
}

fn api_name<'a>(names: &'a [&'a str]) -> impl Fn(&i32) -> String + 'a {
    move |i: &i32| names[usize::try_from(*i).expect("non-negative index")].to_string()
}

// ---------------------------------------------------------------------------
// 1. Create session: Auth + User → Session_Token
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_session_orchestration() {
    let db = setup().await;
    let store = HyperedgeStore::new(&db);

    // sources=[Auth_Service, User_Service] → targets=[Session_Token]
    // middle = [Auth=0, User=1, Token=2]
    let cospan: Cospan<i32> = Cospan::new(vec![0, 1], vec![2], vec![0, 1, 2]);
    let names = ["Auth_Service", "User_Service", "Session_Token"];

    let hub_id = store
        .decompose_cospan(
            &cospan,
            "create_session",
            serde_json::json!({
                "timeout": "30m",
                "requires_mfa": true,
                "rate_limit": 100,
                "version": "v2"
            }),
            api_name(&names),
        )
        .await
        .unwrap();

    // "What inputs does create_session need?"
    let inputs = store.sources(&hub_id).await.unwrap();
    assert_eq!(inputs.len(), 2);
    let input_names: Vec<&str> = inputs.iter().map(|n| n.name.as_str()).collect();
    assert!(input_names.contains(&"Auth_Service"));
    assert!(input_names.contains(&"User_Service"));

    // "What does create_session produce?"
    let outputs = store.targets(&hub_id).await.unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].name, "Session_Token");

    // Verify operation metadata
    let hub = store.get_hub(&hub_id).await.unwrap();
    assert_eq!(hub.properties["timeout"], "30m");
    assert_eq!(hub.properties["requires_mfa"], true);
    assert_eq!(hub.properties["rate_limit"], 100);
}

// ---------------------------------------------------------------------------
// 2. Multi-output operation: Order → [Receipt, Notification, Audit_Log]
// ---------------------------------------------------------------------------

#[tokio::test]
async fn multi_output_operation() {
    let db = setup().await;
    let store = HyperedgeStore::new(&db);

    // sources=[Order_Service] → targets=[Receipt, Notification, Audit_Log]
    // middle = [Order=0, Receipt=1, Notif=2, Audit=3]
    let cospan: Cospan<i32> = Cospan::new(vec![0], vec![1, 2, 3], vec![0, 1, 2, 3]);
    let names = ["Order_Service", "Receipt", "Notification", "Audit_Log"];

    let hub_id = store
        .decompose_cospan(
            &cospan,
            "process_order",
            serde_json::json!({"idempotency_key": "order-123", "async_outputs": true}),
            api_name(&names),
        )
        .await
        .unwrap();

    let hub = store.get_hub(&hub_id).await.unwrap();
    assert_eq!(hub.source_count, 1);
    assert_eq!(hub.target_count, 3);

    let outputs = store.targets(&hub_id).await.unwrap();
    assert_eq!(outputs.len(), 3);
    let output_names: Vec<&str> = outputs.iter().map(|n| n.name.as_str()).collect();
    assert!(output_names.contains(&"Receipt"));
    assert!(output_names.contains(&"Notification"));
    assert!(output_names.contains(&"Audit_Log"));
}

// ---------------------------------------------------------------------------
// 3. Roundtrip reconstruction of API operation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn api_operation_roundtrip() {
    let db = setup().await;
    let store = HyperedgeStore::new(&db);

    let cospan: Cospan<i32> = Cospan::new(vec![0, 1], vec![2], vec![0, 1, 2]);
    let names = ["Auth", "User", "Token"];

    let hub_id = store
        .decompose_cospan(
            &cospan,
            "create_session",
            serde_json::json!({"timeout": "30m"}),
            api_name(&names),
        )
        .await
        .unwrap();

    let reconstructed: Cospan<i32> = store.reconstruct_cospan(&hub_id).await.unwrap();
    assert_eq!(reconstructed.left_to_middle(), cospan.left_to_middle());
    assert_eq!(reconstructed.right_to_middle(), cospan.right_to_middle());
    assert_eq!(reconstructed.middle(), cospan.middle());
}

// ---------------------------------------------------------------------------
// 4. Pairwise API dependencies alongside hyperedge operations
// ---------------------------------------------------------------------------

#[tokio::test]
async fn api_pairwise_and_hyperedge_mixed() {
    let db = setup().await;
    let nodes = NodeStore::new(&db);
    let hyper = HyperedgeStore::new(&db);

    // Pairwise dependency: gateway -> auth_service
    let gateway = nodes
        .create("API_Gateway", "service", vec!["public".into()], serde_json::json!({"port": 443}))
        .await
        .unwrap();
    let _auth = nodes
        .create("Auth_Service", "service", vec!["internal".into()], serde_json::json!({"port": 8081}))
        .await
        .unwrap();

    // Hyperedge operation using the same domain
    let cospan: Cospan<i32> = Cospan::new(vec![0, 1], vec![2], vec![0, 1, 2]);
    let names = ["Gateway", "Auth", "Session"];

    let hub_id = hyper
        .decompose_cospan(&cospan, "session_flow", serde_json::json!({}), api_name(&names))
        .await
        .unwrap();

    // Both exist independently
    let gw = nodes.get(&gateway).await.unwrap();
    assert_eq!(gw.name, "API_Gateway");

    let hub = hyper.get_hub(&hub_id).await.unwrap();
    assert_eq!(hub.kind, "session_flow");
}
