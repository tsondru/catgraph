//! Integration tests for [`catgraph_applied::enriched`].
//!
//! Covers the `EnrichedCategory<V>` trait + `HomMap<O, V>` concrete impl.
//! Lawvere-metric tests are appended in Task 10.

use catgraph_applied::{
    enriched::{EnrichedCategory, HomMap},
    rig::{F64Rig, Tropical, UnitInterval},
};

#[test]
fn hommap_unset_hom_returns_zero() {
    let objects = vec!['a', 'b'];
    let hm: HomMap<char, F64Rig> = HomMap::new(objects);
    assert_eq!(hm.hom(&'a', &'b'), F64Rig(0.0));
}

#[test]
fn hommap_set_hom_roundtrip() {
    let objects = vec!['a', 'b'];
    let mut hm: HomMap<char, F64Rig> = HomMap::new(objects);
    hm.set_hom('a', 'b', F64Rig(3.5));
    assert_eq!(hm.hom(&'a', &'b'), F64Rig(3.5));
    // Asymmetric: b→a still zero.
    assert_eq!(hm.hom(&'b', &'a'), F64Rig(0.0));
}

#[test]
fn id_hom_is_rig_one() {
    let hm: HomMap<char, UnitInterval> = HomMap::new(vec!['a']);
    assert_eq!(hm.id_hom(&'a'), UnitInterval::new(1.0).unwrap());
}

#[test]
fn compose_hom_multiplies_in_rig() {
    let mut hm: HomMap<char, F64Rig> = HomMap::new(vec!['a', 'b', 'c']);
    hm.set_hom('a', 'b', F64Rig(2.0));
    hm.set_hom('b', 'c', F64Rig(3.0));
    // Default compose_hom: hom(a,b) * hom(b,c) = 2 * 3 = 6.
    assert_eq!(hm.compose_hom(&'a', &'b', &'c'), F64Rig(6.0));
}

#[test]
fn tropical_enriched_is_shortest_path() {
    // Tropical (min, +): compose_hom gives a+b (sum of distances).
    // Tropical zero = +∞, so unset homs = +∞.
    let mut hm: HomMap<char, Tropical> = HomMap::new(vec!['a', 'b', 'c']);
    hm.set_hom('a', 'b', Tropical(3.0));
    hm.set_hom('b', 'c', Tropical(4.0));
    assert_eq!(hm.compose_hom(&'a', &'b', &'c'), Tropical(7.0));
}

#[test]
fn objects_iterator_preserves_order() {
    let hm: HomMap<char, F64Rig> = HomMap::new(vec!['x', 'y', 'z']);
    let collected: Vec<char> = hm.objects().collect();
    assert_eq!(collected, vec!['x', 'y', 'z']);
}
