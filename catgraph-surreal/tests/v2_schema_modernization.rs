use catgraph_surreal::init_schema_v2;
use catgraph_surreal::node_store::NodeStore;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

async fn setup() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    init_schema_v2(&db).await.unwrap();
    db
}

#[tokio::test]
async fn fts_node_name_prefix_search() {
    let db = setup().await;
    let ns = NodeStore::new(&db);

    ns.create("hydrogen", "element", vec![], serde_json::json!({}))
        .await
        .unwrap();
    ns.create("helium", "element", vec![], serde_json::json!({}))
        .await
        .unwrap();
    ns.create("oxygen", "element", vec![], serde_json::json!({}))
        .await
        .unwrap();

    // FTS search for "hydrogen" should find the matching node with a positive score.
    let mut result = db
        .query(
            "SELECT name, search::score(1) AS score \
             FROM graph_node WHERE name @1@ $query \
             ORDER BY score DESC",
        )
        .bind(("query", "hydrogen".to_string()))
        .await
        .unwrap();
    let hits: Vec<serde_json::Value> = result.take(0).unwrap();
    assert!(!hits.is_empty(), "FTS should find at least one result for 'hydrogen'");
    assert_eq!(hits[0]["name"], "hydrogen");
}

#[tokio::test]
async fn fts_node_name_no_match() {
    let db = setup().await;
    let ns = NodeStore::new(&db);

    ns.create("hydrogen", "element", vec![], serde_json::json!({}))
        .await
        .unwrap();

    let mut result = db
        .query("SELECT name FROM graph_node WHERE name @@ $query")
        .bind(("query", "zzzznotfound".to_string()))
        .await
        .unwrap();
    let hits: Vec<serde_json::Value> = result.take(0).unwrap();
    assert!(hits.is_empty(), "FTS should find nothing for nonsense query");
}
