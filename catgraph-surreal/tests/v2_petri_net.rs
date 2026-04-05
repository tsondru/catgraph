use rust_decimal::Decimal;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

use catgraph::petri_net::{Marking, PetriNet, Transition};
use catgraph_surreal::init_schema_v2;
use catgraph_surreal::petri_net_store::PetriNetStore;

fn d(n: i64) -> Decimal {
    Decimal::from(n)
}

fn combustion_net() -> PetriNet<char> {
    PetriNet::new(
        vec!['H', 'O', 'W'],
        vec![Transition::new(
            vec![(0, d(2)), (1, d(1))],
            vec![(2, d(2))],
        )],
    )
}

async fn setup() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    init_schema_v2(&db).await.unwrap();
    db
}

#[tokio::test]
async fn save_and_load_roundtrip() {
    let db = setup().await;
    let store = PetriNetStore::new(&db);
    let net = combustion_net();

    let net_id = store.save(&net, "combustion").await.unwrap();
    let loaded: PetriNet<char> = store.load(&net_id).await.unwrap();

    assert_eq!(loaded.place_count(), 3);
    assert_eq!(loaded.transition_count(), 1);
    assert_eq!(loaded.places(), &['H', 'O', 'W']);
    // Verify arc weights match
    assert_eq!(loaded.arc_weight_pre(0, 0), d(2)); // H consumed: weight 2
    assert_eq!(loaded.arc_weight_pre(1, 0), d(1)); // O consumed: weight 1
    assert_eq!(loaded.arc_weight_post(2, 0), d(2)); // W produced: weight 2
    // Verify no spurious arcs
    assert_eq!(loaded.arc_weight_pre(2, 0), Decimal::ZERO);
    assert_eq!(loaded.arc_weight_post(0, 0), Decimal::ZERO);
    assert_eq!(loaded.arc_weight_post(1, 0), Decimal::ZERO);
}

#[tokio::test]
async fn save_and_load_marking() {
    let db = setup().await;
    let store = PetriNetStore::new(&db);
    let net = combustion_net();
    let net_id = store.save(&net, "combustion").await.unwrap();

    let marking = Marking::from_vec(vec![(0, d(4)), (1, d(2))]);
    let marking_id = store
        .save_marking(&net_id, &marking, "initial")
        .await
        .unwrap();
    let loaded = store.load_marking(&marking_id).await.unwrap();

    assert_eq!(loaded.get(0), d(4));
    assert_eq!(loaded.get(1), d(2));
    assert_eq!(loaded.get(2), Decimal::ZERO);
}

#[tokio::test]
async fn delete_net() {
    let db = setup().await;
    let store = PetriNetStore::new(&db);
    let net = combustion_net();
    let net_id = store.save(&net, "combustion").await.unwrap();

    // Save a marking too, to verify cascade delete
    let marking = Marking::from_vec(vec![(0, d(2))]);
    store
        .save_marking(&net_id, &marking, "test")
        .await
        .unwrap();

    store.delete(&net_id).await.unwrap();

    let result: Result<PetriNet<char>, _> = store.load(&net_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn list_nets() {
    let db = setup().await;
    let store = PetriNetStore::new(&db);

    let net1 = combustion_net();
    let net2: PetriNet<char> = PetriNet::new(
        vec!['a', 'b'],
        vec![Transition::new(vec![(0, d(1))], vec![(1, d(1))])],
    );
    store.save(&net1, "combustion").await.unwrap();
    store.save(&net2, "simple").await.unwrap();

    let nets = store.list().await.unwrap();
    assert_eq!(nets.len(), 2);
}

#[tokio::test]
async fn empty_net() {
    let db = setup().await;
    let store = PetriNetStore::new(&db);

    let net: PetriNet<char> = PetriNet::new(vec!['x', 'y'], vec![]);
    let net_id = store.save(&net, "empty_transitions").await.unwrap();
    let loaded: PetriNet<char> = store.load(&net_id).await.unwrap();

    assert_eq!(loaded.place_count(), 2);
    assert_eq!(loaded.transition_count(), 0);
    assert_eq!(loaded.places(), &['x', 'y']);
}

#[tokio::test]
async fn marking_zero_tokens_dropped() {
    let db = setup().await;
    let store = PetriNetStore::new(&db);
    let net = combustion_net();
    let net_id = store.save(&net, "combustion").await.unwrap();

    // Create marking with explicit zero values
    let marking = Marking::from_vec(vec![(0, d(3)), (1, Decimal::ZERO), (2, d(0))]);
    let marking_id = store
        .save_marking(&net_id, &marking, "with_zeros")
        .await
        .unwrap();
    let loaded = store.load_marking(&marking_id).await.unwrap();

    // Only non-zero tokens should survive
    assert_eq!(loaded.get(0), d(3));
    assert_eq!(loaded.get(1), Decimal::ZERO);
    assert_eq!(loaded.get(2), Decimal::ZERO);
    assert_eq!(loaded.tokens().len(), 1);
}
