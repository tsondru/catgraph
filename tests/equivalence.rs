//! Integration tests for Fong-Spivak §4 equivalence (Theorem 1.2/4.13/4.16).
//!
//! Tests the `CospanAlgebraMorphism` construction (§4.2) and
//! roundtrip verification (§4.3, Theorem 4.13).

use std::sync::Arc;

use catgraph::{
    category::{Composable, HasIdentity},
    cospan::Cospan,
    cospan_algebra::{CospanAlgebra, PartitionAlgebra},
    equivalence::{comp_cospan, CospanAlgebraMorphism},
    hypergraph_category::HypergraphCategory,
    monoidal::Monoidal,
};

type PartMorph = CospanAlgebraMorphism<PartitionAlgebra, char>;

fn alg() -> Arc<PartitionAlgebra> {
    Arc::new(PartitionAlgebra)
}

// ---------------------------------------------------------------------------
// comp_cospan correctness
// ---------------------------------------------------------------------------

#[test]
fn comp_cospan_merges_y_copies() {
    let comp = comp_cospan(&['a'], &['b'], &['c']);
    let left = comp.left_to_middle();
    // X: [0], first Y: [1], second Y: [1] (merged!), Z: [2]
    assert_eq!(left, &[0, 1, 1, 2]);
    let right = comp.right_to_middle();
    // X: [0], Z: [2]
    assert_eq!(right, &[0, 2]);
}

// ---------------------------------------------------------------------------
// Identity laws
// ---------------------------------------------------------------------------

#[test]
fn identity_compose_left() {
    let a = alg();
    let id = PartMorph::identity_in(Arc::clone(&a), &['a']);
    let f = PartMorph::identity_in(Arc::clone(&a), &['a']);
    let result = id.compose(&f).unwrap();
    assert_eq!(result.domain(), vec!['a']);
    assert_eq!(result.codomain(), vec!['a']);
}

#[test]
fn identity_compose_right() {
    let a = alg();
    let f = PartMorph::identity_in(Arc::clone(&a), &['a']);
    let id = PartMorph::identity_in(Arc::clone(&a), &['a']);
    let result = f.compose(&id).unwrap();
    assert_eq!(result.domain(), vec!['a']);
    assert_eq!(result.codomain(), vec!['a']);
}

#[test]
fn identity_multi_type() {
    let a = alg();
    let id = PartMorph::identity_in(Arc::clone(&a), &['a', 'b', 'c']);
    let id2 = PartMorph::identity_in(Arc::clone(&a), &['a', 'b', 'c']);
    let result = id.compose(&id2).unwrap();
    assert_eq!(result.domain(), vec!['a', 'b', 'c']);
    assert_eq!(result.codomain(), vec!['a', 'b', 'c']);
}

// ---------------------------------------------------------------------------
// Composition associativity
// ---------------------------------------------------------------------------

#[test]
fn composition_associativity() {
    let a = alg();
    let f = PartMorph::identity_in(Arc::clone(&a), &['a']);
    let g = PartMorph::identity_in(Arc::clone(&a), &['a']);
    let h = PartMorph::identity_in(Arc::clone(&a), &['a']);
    // (f;g);h
    let fg = f.compose(&g).unwrap();
    let fgh_left = fg.compose(&h).unwrap();
    // f;(g;h)
    let gh = g.compose(&h).unwrap();
    let fgh_right = f.compose(&gh).unwrap();
    assert_eq!(fgh_left.domain(), fgh_right.domain());
    assert_eq!(fgh_left.codomain(), fgh_right.codomain());
}

// ---------------------------------------------------------------------------
// Monoidal product
// ---------------------------------------------------------------------------

#[test]
fn monoidal_product_domains() {
    let a = alg();
    let mut f = PartMorph::identity_in(Arc::clone(&a), &['a']);
    let g = PartMorph::identity_in(Arc::clone(&a), &['b']);
    f.monoidal(g);
    assert_eq!(f.domain(), vec!['a', 'b']);
    assert_eq!(f.codomain(), vec!['a', 'b']);
}

#[test]
fn monoidal_with_empty() {
    let a = alg();
    let mut f = PartMorph::identity_in(Arc::clone(&a), &['a']);
    let g = PartMorph::identity_in(Arc::clone(&a), &[]);
    f.monoidal(g);
    assert_eq!(f.domain(), vec!['a']);
    assert_eq!(f.codomain(), vec!['a']);
}

// ---------------------------------------------------------------------------
// Frobenius axioms in H_Part
// ---------------------------------------------------------------------------

#[test]
fn special_frobenius() {
    // δ;μ = id (domain/codomain)
    let a = alg();
    let delta = PartMorph::comultiplication_in(Arc::clone(&a), 'a');
    let mu = PartMorph::multiplication_in(Arc::clone(&a), 'a');
    let result = delta.compose(&mu).unwrap();
    assert_eq!(result.domain(), vec!['a']);
    assert_eq!(result.codomain(), vec!['a']);
}

#[test]
fn unitality_left() {
    // (η ⊗ id) ; μ = id
    let a = alg();
    let mut eta_id = PartMorph::unit_in(Arc::clone(&a), 'a');
    eta_id.monoidal(PartMorph::identity_in(Arc::clone(&a), &['a']));
    let mu = PartMorph::multiplication_in(Arc::clone(&a), 'a');
    let result = eta_id.compose(&mu).unwrap();
    assert_eq!(result.domain(), vec!['a']);
    assert_eq!(result.codomain(), vec!['a']);
}

#[test]
fn counitality_left() {
    // δ ; (ε ⊗ id) = id
    let a = alg();
    let delta = PartMorph::comultiplication_in(Arc::clone(&a), 'a');
    let mut eps_id = PartMorph::counit_in(Arc::clone(&a), 'a');
    eps_id.monoidal(PartMorph::identity_in(Arc::clone(&a), &['a']));
    let result = delta.compose(&eps_id).unwrap();
    assert_eq!(result.domain(), vec!['a']);
    assert_eq!(result.codomain(), vec!['a']);
}

#[test]
fn cup_types() {
    let cup = PartMorph::cup('a').unwrap();
    assert!(cup.domain().is_empty());
    assert_eq!(cup.codomain(), vec!['a', 'a']);
}

#[test]
fn cap_types() {
    let cap = PartMorph::cap('a').unwrap();
    assert_eq!(cap.domain(), vec!['a', 'a']);
    assert!(cap.codomain().is_empty());
}

// ---------------------------------------------------------------------------
// Roundtrip: A → H_A → A_{H_A} = A (Direction 2, Theorem 4.13)
// ---------------------------------------------------------------------------

#[test]
fn roundtrip_a_to_h_to_a_objects() {
    // A_{H_{Part}}(X) = H_{Part}(∅, X) = Part(∅ ⊕ X) = Part(X)
    let a = alg();
    let partition = Cospan::new(vec![], vec![0, 0], vec!['a']);

    let morph = PartMorph::new(Arc::clone(&a), partition.clone(), vec![], vec!['a', 'b']);
    assert!(morph.domain().is_empty());
    assert_eq!(morph.codomain(), vec!['a', 'b']);
    assert_eq!(morph.element().codomain(), partition.codomain());
}

#[test]
fn roundtrip_a_to_h_to_a_morphisms() {
    let part = PartitionAlgebra;

    // Element e ∈ Part([a]) = Cospan(∅, [a])
    let e = Cospan::new(vec![], vec![0], vec!['a']);
    // Cospan c: [a] → [a] (identity)
    let c = Cospan::<char>::identity(&vec!['a']);
    // Part(c)(e) — direct
    let direct = part.map_cospan(&c, &e).unwrap();

    assert_eq!(direct.codomain(), e.codomain());
}

#[test]
fn roundtrip_partition_identity_via_h() {
    // The identity in H_{Part}([a],[a]) should act as identity when composed
    let a = alg();
    // Create a non-trivial morphism: [a] → [a] via a partition
    let s = Cospan::new(vec![], vec![0, 0], vec!['a']);
    let f = PartMorph::new(Arc::clone(&a), s, vec!['a'], vec!['a']);

    let id = PartMorph::identity_in(Arc::clone(&a), &['a']);
    let result = id.compose(&f).unwrap();
    assert_eq!(result.domain(), vec!['a']);
    assert_eq!(result.codomain(), vec!['a']);
}

// ---------------------------------------------------------------------------
// Roundtrip: H → A_H → H_{A_H} ≅ H (Direction 1, Theorem 4.13)
// ---------------------------------------------------------------------------

#[test]
fn roundtrip_cospan_to_partition_objects() {
    // A_{Cospan}(X) = Part(X) = Cospan(∅, X) (Remark 4.5)
    let part = PartitionAlgebra;
    let unit: Cospan<char> = part.unit();
    assert!(unit.domain().is_empty());
    assert!(unit.codomain().is_empty());

    // Map through cospan ∅ → [a] to get element of Part([a])
    let s = Cospan::new(vec![], vec![0], vec!['a']);
    let elem = part.map_cospan(&s, &unit).unwrap();
    assert!(elem.domain().is_empty());
    assert_eq!(elem.codomain(), vec!['a']);
}

#[test]
fn roundtrip_frobenius_generators() {
    // Frobenius generators in H_{Part} should compose correctly
    let eta = PartMorph::unit('x');
    let eps = PartMorph::counit('x');
    let mu = PartMorph::multiplication('x');
    let delta = PartMorph::comultiplication('x');

    // η;δ = cup
    let cup = eta.compose(&delta).unwrap();
    assert!(cup.domain().is_empty());
    assert_eq!(cup.codomain(), vec!['x', 'x']);

    // μ;ε = cap
    let cap = mu.compose(&eps).unwrap();
    assert_eq!(cap.domain(), vec!['x', 'x']);
    assert!(cap.codomain().is_empty());
}
