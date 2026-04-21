//! Integration tests for `Corel<Lambda>` — constructor validation,
//! equivalence-class extraction, and composition preserving joint surjectivity.

mod common;

use catgraph::{
    category::{Composable, HasIdentity},
    corel::Corel,
    cospan::Cospan,
    errors::CatgraphError,
    monoidal::{Monoidal, SymmetricMonoidalMorphism},
};

#[test]
fn new_rejects_non_surjective_middle_larger_than_boundary() {
    // Two boundary entries, three middle vertices — last one uncovered.
    let c = Cospan::new(vec![0], vec![1], vec!['a', 'b', 'c']);
    let result = Corel::new(c);
    assert!(matches!(result, Err(CatgraphError::Corel { .. })));
}

#[test]
fn identity_corel_round_trips() {
    let types = vec!['a', 'b', 'c'];
    let id = Corel::<char>::identity(&types);
    let composed = id.compose(&id).unwrap();
    common::assert_corel_eq(&id, &composed);
}

#[test]
fn compose_preserves_joint_surjectivity() {
    let f = Corel::<char>::new(Cospan::new(vec![0], vec![0, 0], vec!['a'])).unwrap();
    let g = Corel::<char>::new(Cospan::new(vec![0, 0], vec![0], vec!['a'])).unwrap();
    let fg = f.compose(&g).unwrap();
    assert!(fg.as_cospan().is_jointly_surjective());
}

#[test]
fn monoidal_product_preserves_joint_surjectivity() {
    let mut a = Corel::<char>::new(Cospan::new(vec![0], vec![0], vec!['a'])).unwrap();
    let b = Corel::<char>::new(Cospan::new(vec![0], vec![0], vec!['b'])).unwrap();
    a.monoidal(b);
    assert!(a.as_cospan().is_jointly_surjective());
}

#[test]
fn equivalence_classes_count_matches_middle_size() {
    let c = Cospan::new(vec![0, 1, 2], vec![0, 1, 2], vec!['a', 'b', 'c']);
    let corel = Corel::new(c).unwrap();
    assert_eq!(corel.equivalence_classes().len(), 3);
}

#[test]
fn merges_transitive_through_middle() {
    // [0, 1] → [0, 1] with middle ['a', 'a']: two separate classes.
    // Flat layout: dom[0,1] at indices 0,1; middle[0,1] at 2,3; cod[0,1] at 4,5.
    let c = Cospan::new(vec![0, 1], vec![0, 1], vec!['a', 'a']);
    let corel = Corel::new(c).unwrap();
    assert!(corel.merges(0, 2));  // dom[0] <-> middle[0]
    assert!(corel.merges(0, 4));  // dom[0] <-> cod[0]
    assert!(!corel.merges(0, 1)); // dom[0] != dom[1]
}

#[test]
fn refines_rejects_shape_mismatch() {
    let a = Corel::<char>::new(Cospan::new(vec![0], vec![0], vec!['a'])).unwrap();
    let b = Corel::<char>::new(Cospan::new(vec![0, 0], vec![0, 0], vec!['a'])).unwrap();
    assert!(matches!(a.refines(&b), Err(CatgraphError::Corel { .. })));
}

#[test]
fn ccr_rejects_shape_mismatch() {
    let a = Corel::<char>::new(Cospan::new(vec![0], vec![0], vec!['a'])).unwrap();
    let b = Corel::<char>::new(Cospan::new(vec![0, 0], vec![0, 0], vec!['a'])).unwrap();
    assert!(matches!(
        a.coarsest_common_refinement(&b),
        Err(CatgraphError::Corel { .. })
    ));
}

#[test]
fn symmetric_braiding_preserves_surjectivity() {
    let braid = Corel::<char>::from_permutation(
        permutations::Permutation::transposition(2, 0, 1),
        &['a', 'b'],
        true,
    )
    .unwrap();
    assert!(braid.as_cospan().is_jointly_surjective());
}
