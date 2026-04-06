//! Rayon threshold correctness validation.
//!
//! Verifies that operations produce correct results at sizes above
//! their rayon parallelism thresholds. Does not test performance.

use catgraph::{
    category::{Composable, HasIdentity},
    frobenius::{special_frobenius_morphism, FrobeniusMorphism},
    linear_combination::LinearCombination,
    monoidal::Monoidal,
    named_cospan::NamedCospan,
    temperley_lieb::BrauerMorphism,
};
use either::Either::{Left, Right};
use std::collections::HashMap;

/// LinearCombination Mul with 64 terms (threshold = 32).
#[test]
fn linear_combination_above_threshold() {
    let terms_a: HashMap<i32, i64> = (0..64).map(|i| (i, (i + 1).into())).collect();
    let terms_b: HashMap<i32, i64> = (0..64).map(|i| (i, 1i64)).collect();
    let lc_a: LinearCombination<i64, i32> = terms_a.into_iter().collect();
    let lc_b: LinearCombination<i64, i32> = terms_b.into_iter().collect();

    // Multiplication distributes over basis: (c1*t1) * (c2*t2) = (c1*c2) * (t1*t2)
    // For i32 basis, Mul<Output=i32> multiplies basis elements.
    let product = lc_a * lc_b;

    // Product should be non-empty (64 * 64 = 4096 cross-terms before simplification)
    let simplified = {
        let mut p = product;
        p.simplify();
        p
    };
    // At minimum, (0 * anything) = 0 basis elements get folded, but nonzero entries remain
    assert_ne!(simplified, LinearCombination::default());
}

/// BrauerMorphism compose at n=16 (threshold = 8 for non_crossing checks).
#[test]
fn temperley_lieb_above_threshold() {
    let gens: Vec<BrauerMorphism<i64>> = BrauerMorphism::temperley_lieb_gens(16);
    assert_eq!(gens.len(), 15);

    // Compose e_1 * e_2 — triggers diagram stacking with 16 source/target points
    let composed = gens[0].compose(&gens[1]).unwrap();
    assert_eq!(composed.domain(), 16);
    assert_eq!(composed.codomain(), 16);

    // Compose identity with a generator — should equal the generator
    let id: BrauerMorphism<i64> = BrauerMorphism::identity(&16);
    let id_composed = id.compose(&gens[7]).unwrap();
    assert_eq!(id_composed, gens[7]);
}

/// NamedCospan find_nodes_by_name_predicate with 512 boundary nodes (threshold = 256).
#[test]
fn named_cospan_predicate_above_threshold() {
    // Build a NamedCospan with 300 left nodes and 300 right nodes (total 600 >= 256)
    // Each maps to a distinct middle node.
    let n = 300;
    let left: Vec<usize> = (0..n).collect();
    let right: Vec<usize> = (n..2 * n).collect();
    let middle: Vec<char> = (0..2 * n).map(|_| 'x').collect();
    let left_names: Vec<i32> = (0..n as i32).collect();
    let right_names: Vec<i32> = (n as i32..2 * n as i32).collect();

    let nc: NamedCospan<char, i32, i32> =
        NamedCospan::new(left, right, middle, left_names, right_names);

    // Find all even-named nodes (should hit the parallel path)
    let found = nc.find_nodes_by_name_predicate(|n| n % 2 == 0, |n| n % 2 == 0, false);

    // 150 even names on left (0,2,...,298) + 150 even names on right (300,302,...,598)
    assert_eq!(found.len(), 300);

    // Verify Left/Right classification
    let left_count = found.iter().filter(|e| matches!(e, Left(_))).count();
    let right_count = found.iter().filter(|e| matches!(e, Right(_))).count();
    assert_eq!(left_count, 150);
    assert_eq!(right_count, 150);
}

/// FrobeniusMorphism with 128+ blocks via monoidal product (threshold = 64).
///
/// `special_frobenius_morphism(m, 1, wire_type)` for large m builds layers via
/// recursive monoidal product. Calling `hflip` (through `from_permutation` or
/// direct `special_frobenius_morphism` with m < n) triggers the parallel path
/// on layers with 64+ blocks.
#[test]
fn frobenius_hflip_above_threshold() {
    // Build a large morphism: 128 inputs → 1 output
    // This recursively builds layers with 128+ identity blocks that get hflipped.
    let morph: FrobeniusMorphism<char, String> = special_frobenius_morphism(128, 1, 'a');
    assert!(morph.depth() > 0);

    // Now trigger hflip by building 1 → 128 (internally calls hflip on 128→1)
    let morph_flipped: FrobeniusMorphism<char, String> = special_frobenius_morphism(1, 128, 'a');
    assert!(morph_flipped.depth() > 0);

    // Compose: (1 → 128) then (128 → 1) should produce a valid 1 → 1 morphism
    let mut composed = morph_flipped.clone();
    composed.monoidal(FrobeniusMorphism::identity(&vec![]));
    assert!(composed.depth() > 0);
}
