use catgraph::named_cospan::NamedCospan;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

use catgraph_surreal::init_schema;
use catgraph_surreal::named_cospan_store::NamedCospanStore;

async fn setup() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    init_schema(&db).await.unwrap();
    db
}

#[tokio::test]
async fn roundtrip_named_cospan() {
    let db = setup().await;
    let store = NamedCospanStore::new(&db);

    // Named cospan with string port names
    // left: ["input_a", "input_b"] -> middle: ['x', 'y']
    // right: ["output_c"] -> middle node 1
    let nc = NamedCospan::<char, String, String>::new(
        vec![0, 1],
        vec![1],
        vec!['x', 'y'],
        vec!["input_a".into(), "input_b".into()],
        vec!["output_c".into()],
    );

    let id = store.save(&nc).await.unwrap();
    let loaded = store.load::<char>(&id).await.unwrap();

    assert_eq!(loaded.cospan().left_to_middle(), nc.cospan().left_to_middle());
    assert_eq!(loaded.cospan().right_to_middle(), nc.cospan().right_to_middle());
    assert_eq!(loaded.cospan().middle(), nc.cospan().middle());
    assert_eq!(loaded.left_names(), nc.left_names());
    assert_eq!(loaded.right_names(), nc.right_names());
}

#[tokio::test]
async fn empty_named_cospan() {
    let db = setup().await;
    let store = NamedCospanStore::new(&db);

    let nc = NamedCospan::<char, String, String>::empty();

    let id = store.save(&nc).await.unwrap();
    let loaded = store.load::<char>(&id).await.unwrap();

    assert!(loaded.left_names().is_empty());
    assert!(loaded.right_names().is_empty());
    assert!(loaded.cospan().middle().is_empty());
}

#[tokio::test]
async fn delete_named_cospan() {
    let db = setup().await;
    let store = NamedCospanStore::new(&db);

    let nc = NamedCospan::<char, String, String>::new(
        vec![0],
        vec![0],
        vec!['a'],
        vec!["in".into()],
        vec!["out".into()],
    );

    let id = store.save(&nc).await.unwrap();
    store.delete(&id).await.unwrap();

    let result = store.load::<char>(&id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn list_named_cospans() {
    let db = setup().await;
    let store = NamedCospanStore::new(&db);

    let nc1 = NamedCospan::<char, String, String>::new(
        vec![0],
        vec![0],
        vec!['a'],
        vec!["p1".into()],
        vec!["q1".into()],
    );
    let nc2 = NamedCospan::<char, String, String>::new(
        vec![0],
        vec![0],
        vec!['b'],
        vec!["p2".into()],
        vec!["q2".into()],
    );

    store.save(&nc1).await.unwrap();
    store.save(&nc2).await.unwrap();

    let ids = store.list().await.unwrap();
    assert_eq!(ids.len(), 2);
}

#[tokio::test]
async fn port_names_preserved_exactly() {
    let db = setup().await;
    let store = NamedCospanStore::new(&db);

    let nc = NamedCospan::<char, String, String>::new(
        vec![0, 1, 0],
        vec![1, 0],
        vec!['x', 'y'],
        vec!["alpha".into(), "beta".into(), "gamma".into()],
        vec!["delta".into(), "epsilon".into()],
    );

    let id = store.save(&nc).await.unwrap();
    let loaded = store.load::<char>(&id).await.unwrap();

    assert_eq!(loaded.left_names(), &vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()]);
    assert_eq!(loaded.right_names(), &vec!["delta".to_string(), "epsilon".to_string()]);
}
