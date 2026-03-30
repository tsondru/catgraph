use catgraph::category::{Composable, HasIdentity};
use catgraph::span::Span;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

use catgraph_surreal::init_schema;
use catgraph_surreal::span_store::SpanStore;

async fn setup() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    init_schema(&db).await.unwrap();
    db
}

#[tokio::test]
async fn roundtrip_char_span() {
    let db = setup().await;
    let store = SpanStore::new(&db);

    // Span: left=[a,b], right=[a,b], middle=[(0,0),(1,1)]
    let span = Span::new(vec!['a', 'b'], vec!['a', 'b'], vec![(0, 0), (1, 1)]);

    let id = store.save::<char>(&span).await.unwrap();
    let loaded: Span<char> = store.load(&id).await.unwrap();

    assert_eq!(loaded.left(), span.left());
    assert_eq!(loaded.right(), span.right());
    assert_eq!(loaded.middle_pairs(), span.middle_pairs());
    assert_eq!(loaded.is_left_identity(), span.is_left_identity());
    assert_eq!(loaded.is_right_identity(), span.is_right_identity());
}

#[tokio::test]
async fn roundtrip_unit_span() {
    let db = setup().await;
    let store = SpanStore::new(&db);

    let span = Span::new(vec![(), ()], vec![(), ()], vec![(0, 0), (1, 1)]);

    let id = store.save::<()>(&span).await.unwrap();
    let loaded: Span<()> = store.load(&id).await.unwrap();

    assert_eq!(loaded.left(), span.left());
    assert_eq!(loaded.right(), span.right());
    assert_eq!(loaded.middle_pairs(), span.middle_pairs());
}

#[tokio::test]
async fn span_identity_preservation() {
    let db = setup().await;
    let store = SpanStore::new(&db);

    let id_span = Span::<char>::identity(&vec!['x', 'y', 'z']);
    assert!(id_span.is_left_identity());
    assert!(id_span.is_right_identity());

    let id = store.save(&id_span).await.unwrap();
    let loaded: Span<char> = store.load(&id).await.unwrap();

    assert!(loaded.is_left_identity());
    assert!(loaded.is_right_identity());
    assert_eq!(loaded.middle_pairs(), id_span.middle_pairs());
}

#[tokio::test]
async fn span_non_identity_right() {
    let db = setup().await;
    let store = SpanStore::new(&db);

    // left identity (middle_to_left = [0,1]) but NOT right identity (middle_to_right = [0,0])
    let span = Span::new(vec!['a', 'a'], vec!['a'], vec![(0, 0), (1, 0)]);

    assert!(span.is_left_identity());
    assert!(!span.is_right_identity());

    let id = store.save(&span).await.unwrap();
    let loaded: Span<char> = store.load(&id).await.unwrap();

    assert!(loaded.is_left_identity());
    assert!(!loaded.is_right_identity());
    assert_eq!(loaded.middle_pairs(), span.middle_pairs());
}

#[tokio::test]
async fn delete_span() {
    let db = setup().await;
    let store = SpanStore::new(&db);

    let span = Span::new(vec!['a'], vec!['a'], vec![(0, 0)]);
    let id = store.save(&span).await.unwrap();

    store.delete(&id).await.unwrap();
    let result: Result<Span<char>, _> = store.load(&id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn list_spans() {
    let db = setup().await;
    let store = SpanStore::new(&db);

    let s1 = Span::new(vec!['a'], vec!['a'], vec![(0, 0)]);
    let s2 = Span::new(vec!['b'], vec!['b'], vec![(0, 0)]);

    store.save::<char>(&s1).await.unwrap();
    store.save::<char>(&s2).await.unwrap();

    let ids = store.list().await.unwrap();
    assert_eq!(ids.len(), 2);
}

#[tokio::test]
async fn span_compose_then_persist() {
    let db = setup().await;
    let store = SpanStore::new(&db);

    let f = Span::new(vec!['a', 'b'], vec!['a', 'b'], vec![(0, 0), (1, 1)]);
    let g = Span::new(vec!['a', 'b'], vec!['a', 'b'], vec![(0, 0), (1, 1)]);

    let composed = f.compose(&g).unwrap();
    let id = store.save(&composed).await.unwrap();
    let loaded: Span<char> = store.load(&id).await.unwrap();

    assert_eq!(loaded.left(), composed.left());
    assert_eq!(loaded.right(), composed.right());
    assert_eq!(loaded.middle_pairs(), composed.middle_pairs());
}

#[tokio::test]
async fn span_dagger_roundtrip() {
    let db = setup().await;
    let store = SpanStore::new(&db);

    let span = Span::new(vec!['a', 'b'], vec!['a', 'b'], vec![(0, 0), (1, 1)]);
    let dagger = span.dagger();

    let id = store.save(&dagger).await.unwrap();
    let loaded: Span<char> = store.load(&id).await.unwrap();

    assert_eq!(loaded.left(), dagger.left());
    assert_eq!(loaded.right(), dagger.right());
    assert_eq!(loaded.middle_pairs(), dagger.middle_pairs());
}
