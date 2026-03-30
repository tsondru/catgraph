use catgraph::category::{Composable, HasIdentity};
use catgraph::cospan::Cospan;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

use catgraph_surreal::cospan_store::CospanStore;
use catgraph_surreal::init_schema;

async fn setup() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    init_schema(&db).await.unwrap();
    db
}

#[tokio::test]
async fn roundtrip_char_cospan() {
    let db = setup().await;
    let store = CospanStore::new(&db);

    // Cospan: {a,b} -> {x,y,z} <- {c}
    // left maps: a->x(0), b->y(1)
    // right maps: c->z(2)
    let cospan = Cospan::new(vec![0, 1], vec![2], vec!['x', 'y', 'z']);

    let id = store.save::<char>(&cospan).await.unwrap();
    let loaded: Cospan<char> = store.load(&id).await.unwrap();

    assert_eq!(loaded.left_to_middle(), cospan.left_to_middle());
    assert_eq!(loaded.right_to_middle(), cospan.right_to_middle());
    assert_eq!(loaded.middle(), cospan.middle());
    assert_eq!(loaded.is_left_identity(), cospan.is_left_identity());
    assert_eq!(loaded.is_right_identity(), cospan.is_right_identity());
}

#[tokio::test]
async fn roundtrip_unit_cospan() {
    let db = setup().await;
    let store = CospanStore::new(&db);

    // Unit-typed cospan: both sides map to a single middle node
    let cospan = Cospan::new(vec![0], vec![0], vec![()]);

    let id = store.save::<()>(&cospan).await.unwrap();
    let loaded: Cospan<()> = store.load(&id).await.unwrap();

    assert_eq!(loaded.left_to_middle(), cospan.left_to_middle());
    assert_eq!(loaded.right_to_middle(), cospan.right_to_middle());
    assert_eq!(loaded.middle(), cospan.middle());
}

#[tokio::test]
async fn identity_preservation() {
    let db = setup().await;
    let store = CospanStore::new(&db);

    let id_cospan = Cospan::<char>::identity(&vec!['a', 'b', 'c']);

    assert!(id_cospan.is_left_identity());
    assert!(id_cospan.is_right_identity());

    let id = store.save(&id_cospan).await.unwrap();
    let loaded: Cospan<char> = store.load(&id).await.unwrap();

    assert!(loaded.is_left_identity());
    assert!(loaded.is_right_identity());
}

#[tokio::test]
async fn empty_cospan() {
    let db = setup().await;
    let store = CospanStore::new(&db);

    let cospan = Cospan::<char>::empty();

    let id = store.save(&cospan).await.unwrap();
    let loaded: Cospan<char> = store.load(&id).await.unwrap();

    assert!(loaded.middle().is_empty());
    assert!(loaded.left_to_middle().is_empty());
    assert!(loaded.right_to_middle().is_empty());
}

#[tokio::test]
async fn single_node_cospan() {
    let db = setup().await;
    let store = CospanStore::new(&db);

    // Single middle node, both sides map to it
    let cospan = Cospan::new(vec![0, 0], vec![0], vec!['x']);

    let id = store.save(&cospan).await.unwrap();
    let loaded: Cospan<char> = store.load(&id).await.unwrap();

    assert_eq!(loaded.left_to_middle(), &[0, 0]);
    assert_eq!(loaded.right_to_middle(), &[0]);
    assert_eq!(loaded.middle(), &['x']);
}

#[tokio::test]
async fn delete_cospan() {
    let db = setup().await;
    let store = CospanStore::new(&db);

    let cospan = Cospan::new(vec![0], vec![0], vec!['a']);
    let id = store.save(&cospan).await.unwrap();

    // Verify it exists
    let loaded: Cospan<char> = store.load(&id).await.unwrap();
    assert_eq!(loaded.middle(), &['a']);

    // Delete and verify gone
    store.delete(&id).await.unwrap();
    let result: Result<Cospan<char>, _> = store.load(&id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn list_cospans() {
    let db = setup().await;
    let store = CospanStore::new(&db);

    let c1 = Cospan::new(vec![0], vec![0], vec!['a']);
    let c2 = Cospan::new(vec![0], vec![0], vec!['b']);

    store.save::<char>(&c1).await.unwrap();
    store.save::<char>(&c2).await.unwrap();

    let ids = store.list().await.unwrap();
    assert_eq!(ids.len(), 2);
}

#[tokio::test]
async fn compose_then_persist() {
    let db = setup().await;
    let store = CospanStore::new(&db);

    // f: {a,b} -> {x,y} (identity-like)
    let f = Cospan::<char>::new(vec![0, 1], vec![0, 1], vec!['a', 'b']);
    // g: {a,b} -> {x,y} (identity-like)
    let g = Cospan::<char>::new(vec![0, 1], vec![0, 1], vec!['a', 'b']);

    let composed = f.compose(&g).unwrap();
    let id = store.save(&composed).await.unwrap();
    let loaded: Cospan<char> = store.load(&id).await.unwrap();

    assert_eq!(loaded.left_to_middle(), composed.left_to_middle());
    assert_eq!(loaded.right_to_middle(), composed.right_to_middle());
    assert_eq!(loaded.middle(), composed.middle());
}

#[tokio::test]
async fn type_mismatch_error() {
    let db = setup().await;
    let store = CospanStore::new(&db);

    let cospan = Cospan::new(vec![0], vec![0], vec!['a']);
    let id = store.save::<char>(&cospan).await.unwrap();

    // Try to load as unit type — should fail with type mismatch
    let result: Result<Cospan<()>, _> = store.load(&id).await;
    assert!(result.is_err());
}
