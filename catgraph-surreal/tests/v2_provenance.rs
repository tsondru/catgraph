use catgraph::cospan::Cospan;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;
use surrealdb_types::SurrealValue;

use catgraph_surreal::hyperedge_store::HyperedgeStore;
use catgraph_surreal::init_schema_v2;

async fn setup_v2() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    init_schema_v2(&db).await.unwrap();
    db
}

// ---------------------------------------------------------------------------
// 1. Provenance roundtrip: parent hub recorded and queryable
// ---------------------------------------------------------------------------

#[tokio::test]
async fn provenance_roundtrip() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    // Create cospan A (no provenance)
    let cospan_a = Cospan::new(vec![0, 1], vec![2], vec!['x', 'y', 'z']);
    let hub_a = store
        .decompose_cospan(&cospan_a, "base", serde_json::json!({}), |c| c.to_string())
        .await
        .unwrap();

    // Create cospan B with provenance pointing to hub_a
    let cospan_b = Cospan::new(vec![0], vec![0], vec!['m']);
    let hub_b = store
        .decompose_cospan_with_provenance(
            &cospan_b,
            "composed",
            serde_json::json!({}),
            |c| c.to_string(),
            &[hub_a.clone()],
        )
        .await
        .unwrap();

    // composition_parents(hub_b) should return the string form of hub_a
    let parents = store.composition_parents(&hub_b).await.unwrap();
    assert_eq!(parents.len(), 1, "hub_b should have exactly 1 parent");
    // The parent string should contain "hyperedge_hub:" prefix
    assert!(
        parents[0].starts_with("hyperedge_hub:"),
        "parent string should be formatted as table:key, got: {}",
        parents[0]
    );

    // composition_children(hub_a) should return hub_b
    let children = store.composition_children(&hub_a).await.unwrap();
    assert_eq!(children.len(), 1, "hub_a should have exactly 1 child");
    assert_eq!(
        children[0].id.as_ref().unwrap(),
        &hub_b,
        "child should be hub_b"
    );
}

// ---------------------------------------------------------------------------
// 2. Provenance chain: A -> B -> C
// ---------------------------------------------------------------------------

#[tokio::test]
async fn provenance_chain() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    // Hub A — no parents
    let hub_a = store
        .decompose_cospan(
            &Cospan::new(vec![0], vec![0], vec!['a']),
            "chain",
            serde_json::json!({}),
            |c| c.to_string(),
        )
        .await
        .unwrap();

    // Hub B — parent = A
    let hub_b = store
        .decompose_cospan_with_provenance(
            &Cospan::new(vec![0], vec![0], vec!['b']),
            "chain",
            serde_json::json!({}),
            |c| c.to_string(),
            &[hub_a.clone()],
        )
        .await
        .unwrap();

    // Hub C — parent = B
    let hub_c = store
        .decompose_cospan_with_provenance(
            &Cospan::new(vec![0], vec![0], vec!['c']),
            "chain",
            serde_json::json!({}),
            |c| c.to_string(),
            &[hub_b.clone()],
        )
        .await
        .unwrap();

    // composition_children(A) = [B]
    let children_a = store.composition_children(&hub_a).await.unwrap();
    assert_eq!(children_a.len(), 1);
    assert_eq!(children_a[0].id.as_ref().unwrap(), &hub_b);

    // composition_children(B) = [C]
    let children_b = store.composition_children(&hub_b).await.unwrap();
    assert_eq!(children_b.len(), 1);
    assert_eq!(children_b[0].id.as_ref().unwrap(), &hub_c);

    // composition_parents(C) = [B]
    let parents_c = store.composition_parents(&hub_c).await.unwrap();
    assert_eq!(parents_c.len(), 1);

    // composition_parents(A) = []
    let parents_a = store.composition_parents(&hub_a).await.unwrap();
    assert!(parents_a.is_empty(), "root hub should have no parents");
}

// ---------------------------------------------------------------------------
// 3. No provenance — regular decompose yields empty parents
// ---------------------------------------------------------------------------

#[tokio::test]
async fn no_provenance_returns_empty() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    let hub_id = store
        .decompose_cospan(
            &Cospan::new(vec![0], vec![0], vec!['z']),
            "plain",
            serde_json::json!({}),
            |c| c.to_string(),
        )
        .await
        .unwrap();

    let parents = store.composition_parents(&hub_id).await.unwrap();
    assert!(parents.is_empty(), "hub without provenance should have no parents");

    let children = store.composition_children(&hub_id).await.unwrap();
    assert!(children.is_empty(), "hub with no descendants should have no children");
}

// ---------------------------------------------------------------------------
// 4. Multiple parents — diamond provenance
// ---------------------------------------------------------------------------

#[tokio::test]
async fn provenance_multiple_parents() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    // Two base hubs
    let hub_a = store
        .decompose_cospan(
            &Cospan::new(vec![0], vec![0], vec!['a']),
            "base",
            serde_json::json!({}),
            |c| c.to_string(),
        )
        .await
        .unwrap();

    let hub_b = store
        .decompose_cospan(
            &Cospan::new(vec![0], vec![0], vec!['b']),
            "base",
            serde_json::json!({}),
            |c| c.to_string(),
        )
        .await
        .unwrap();

    // Hub C composed from both A and B
    let hub_c = store
        .decompose_cospan_with_provenance(
            &Cospan::new(vec![0, 1], vec![0], vec!['x', 'y']),
            "merged",
            serde_json::json!({}),
            |c| c.to_string(),
            &[hub_a.clone(), hub_b.clone()],
        )
        .await
        .unwrap();

    // composition_parents(C) = [A, B]
    let parents = store.composition_parents(&hub_c).await.unwrap();
    assert_eq!(parents.len(), 2, "hub_c should have 2 parents");

    // composition_children(A) = [C]
    let children_a = store.composition_children(&hub_a).await.unwrap();
    assert_eq!(children_a.len(), 1);
    assert_eq!(children_a[0].id.as_ref().unwrap(), &hub_c);

    // composition_children(B) = [C]
    let children_b = store.composition_children(&hub_b).await.unwrap();
    assert_eq!(children_b.len(), 1);
    assert_eq!(children_b[0].id.as_ref().unwrap(), &hub_c);
}

// ---------------------------------------------------------------------------
// 5. Provenance preserves custom properties
// ---------------------------------------------------------------------------

#[tokio::test]
async fn provenance_preserves_custom_properties() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    let hub_a = store
        .decompose_cospan(
            &Cospan::new(vec![0], vec![0], vec!['a']),
            "base",
            serde_json::json!({}),
            |c| c.to_string(),
        )
        .await
        .unwrap();

    // Pass custom properties alongside provenance
    let hub_b = store
        .decompose_cospan_with_provenance(
            &Cospan::new(vec![0], vec![0], vec!['b']),
            "annotated",
            serde_json::json!({"description": "composed from A", "version": 2}),
            |c| c.to_string(),
            &[hub_a.clone()],
        )
        .await
        .unwrap();

    let hub = store.get_hub(&hub_b).await.unwrap();
    assert_eq!(hub.properties["description"], "composed from A");
    assert_eq!(hub.properties["version"], 2);
    assert!(hub.properties["parent_hubs"].is_array());
}

// ---------------------------------------------------------------------------
// 6. Cospan data integrity after provenance decompose
// ---------------------------------------------------------------------------

#[tokio::test]
async fn provenance_cospan_data_intact() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    let parent = store
        .decompose_cospan(
            &Cospan::new(vec![0], vec![0], vec!['p']),
            "parent",
            serde_json::json!({}),
            |c| c.to_string(),
        )
        .await
        .unwrap();

    let cospan = Cospan::new(vec![0, 1], vec![2], vec!['x', 'y', 'z']);
    let hub_id = store
        .decompose_cospan_with_provenance(
            &cospan,
            "child",
            serde_json::json!({}),
            |c| c.to_string(),
            &[parent],
        )
        .await
        .unwrap();

    // Reconstruct should still work correctly
    let reconstructed: Cospan<char> = store.reconstruct_cospan(&hub_id).await.unwrap();
    assert_eq!(reconstructed.left_to_middle(), cospan.left_to_middle());
    assert_eq!(reconstructed.right_to_middle(), cospan.right_to_middle());
    assert_eq!(reconstructed.middle(), cospan.middle());
}

// ===========================================================================
// Schema-level provenance features (merged from v2_schema_enhancements.rs)
// ===========================================================================

// ---------------------------------------------------------------------------
// 7. Record reference: parent_hubs field contains parent RecordId directly
// ---------------------------------------------------------------------------

#[tokio::test]
async fn record_reference_provenance() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    let cospan_a = Cospan::new(vec![0, 1], vec![2], vec!['x', 'y', 'z']);
    let hub_a = store
        .decompose_cospan(&cospan_a, "base", serde_json::json!({}), |c| c.to_string())
        .await
        .unwrap();

    let cospan_b = Cospan::new(vec![0], vec![0], vec!['m']);
    let hub_b = store
        .decompose_cospan_with_provenance(
            &cospan_b,
            "composed",
            serde_json::json!({}),
            |c| c.to_string(),
            &[hub_a.clone()],
        )
        .await
        .unwrap();

    let hub_b_record = store.get_hub(&hub_b).await.unwrap();
    let parent_hubs = hub_b_record
        .parent_hubs
        .expect("hub_b should have parent_hubs set");
    assert_eq!(parent_hubs.len(), 1);
    assert_eq!(parent_hubs[0], hub_a);
}

// ---------------------------------------------------------------------------
// 8. ON DELETE UNSET: deleting a parent clears it from child's parent_hubs
// ---------------------------------------------------------------------------

#[tokio::test]
async fn on_delete_unset_cascade() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    let hub_a = store
        .decompose_cospan(
            &Cospan::new(vec![0], vec![0], vec!['a']),
            "base",
            serde_json::json!({}),
            |c| c.to_string(),
        )
        .await
        .unwrap();

    let hub_b = store
        .decompose_cospan_with_provenance(
            &Cospan::new(vec![0], vec![0], vec!['b']),
            "child",
            serde_json::json!({}),
            |c| c.to_string(),
            &[hub_a.clone()],
        )
        .await
        .unwrap();

    let before = store.get_hub(&hub_b).await.unwrap();
    assert_eq!(before.parent_hubs.as_ref().map(|v| v.len()), Some(1));

    store.delete_hub(&hub_a).await.unwrap();

    let after = store.get_hub(&hub_b).await.unwrap();
    let remaining = after.parent_hubs.unwrap_or_default();
    assert!(remaining.is_empty(), "parent_hubs should be empty after parent deletion");
}

// ---------------------------------------------------------------------------
// 9. composed_from RELATE edge: track inbound composition relations
// ---------------------------------------------------------------------------

#[tokio::test]
async fn composed_from_relation() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    let hub_a = store
        .decompose_cospan(
            &Cospan::new(vec![0], vec![0], vec!['a']),
            "base",
            serde_json::json!({}),
            |c| c.to_string(),
        )
        .await
        .unwrap();

    let hub_b = store
        .decompose_cospan(
            &Cospan::new(vec![0], vec![0], vec!['b']),
            "base",
            serde_json::json!({}),
            |c| c.to_string(),
        )
        .await
        .unwrap();

    let hub_c = store
        .decompose_cospan(
            &Cospan::new(vec![0], vec![0], vec!['c']),
            "result",
            serde_json::json!({}),
            |c| c.to_string(),
        )
        .await
        .unwrap();

    let rel_a = store.relate_composition(&hub_a, &hub_c, "pushout").await.unwrap();
    let rel_b = store.relate_composition(&hub_b, &hub_c, "pushout").await.unwrap();
    assert_ne!(rel_a, rel_b);

    let mut result = db
        .query("SELECT <-composed_from<-hyperedge_hub AS parents FROM $hub")
        .bind(("hub", hub_c.clone()))
        .await
        .unwrap();

    #[derive(Debug, serde::Deserialize, SurrealValue)]
    struct ParentsResult {
        parents: Vec<surrealdb::types::RecordId>,
    }
    let rows: Vec<ParentsResult> = result.take(0).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].parents.len(), 2);
    assert!(rows[0].parents.contains(&hub_a));
    assert!(rows[0].parents.contains(&hub_b));
}

// ---------------------------------------------------------------------------
// 10. has_provenance computed field
// ---------------------------------------------------------------------------

#[tokio::test]
async fn has_provenance_computed() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    let hub_no_parents = store
        .decompose_cospan(
            &Cospan::new(vec![0], vec![0], vec!['n']),
            "plain",
            serde_json::json!({}),
            |c| c.to_string(),
        )
        .await
        .unwrap();

    let hub_with_parents = store
        .decompose_cospan_with_provenance(
            &Cospan::new(vec![0], vec![0], vec!['p']),
            "composed",
            serde_json::json!({}),
            |c| c.to_string(),
            &[hub_no_parents.clone()],
        )
        .await
        .unwrap();

    #[derive(Debug, serde::Deserialize, SurrealValue)]
    struct ProvenanceCheck {
        has_provenance: Option<bool>,
    }

    let mut result = db
        .query("SELECT has_provenance FROM $hub")
        .bind(("hub", hub_no_parents))
        .await
        .unwrap();
    let rows: Vec<ProvenanceCheck> = result.take(0).unwrap();
    assert_eq!(rows[0].has_provenance, Some(false));

    let mut result = db
        .query("SELECT has_provenance FROM $hub")
        .bind(("hub", hub_with_parents))
        .await
        .unwrap();
    let rows: Vec<ProvenanceCheck> = result.take(0).unwrap();
    assert_eq!(rows[0].has_provenance, Some(true));
}

// ---------------------------------------------------------------------------
// 11. composed_children_via_ref: query children through REFERENCE field
// ---------------------------------------------------------------------------

#[tokio::test]
async fn composed_children_via_ref() {
    let db = setup_v2().await;
    let store = HyperedgeStore::new(&db);

    let hub_a = store
        .decompose_cospan(
            &Cospan::new(vec![0], vec![0], vec!['a']),
            "base",
            serde_json::json!({}),
            |c| c.to_string(),
        )
        .await
        .unwrap();

    let hub_b = store
        .decompose_cospan_with_provenance(
            &Cospan::new(vec![0], vec![0], vec!['b']),
            "child",
            serde_json::json!({}),
            |c| c.to_string(),
            &[hub_a.clone()],
        )
        .await
        .unwrap();

    let hub_c = store
        .decompose_cospan_with_provenance(
            &Cospan::new(vec![0, 1], vec![0], vec!['c', 'd']),
            "child",
            serde_json::json!({}),
            |c| c.to_string(),
            &[hub_a.clone()],
        )
        .await
        .unwrap();

    let children = store.composed_children_via_ref(&hub_a).await.unwrap();
    assert_eq!(children.len(), 2);
    let child_ids: Vec<_> = children.iter().filter_map(|h| h.id.as_ref()).collect();
    assert!(child_ids.contains(&&hub_b));
    assert!(child_ids.contains(&&hub_c));

    let no_children = store.composed_children_via_ref(&hub_b).await.unwrap();
    assert!(no_children.is_empty());
}
