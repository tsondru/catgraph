//! Integration tests for cospan-algebras (Fong-Spivak §2.1).
//!
//! Tests the CospanAlgebra trait, PartitionAlgebra (Example 2.3),
//! and NameAlgebra (§4.1) via the public API.

use catgraph::{
    category::{Composable, ComposableMutating, HasIdentity},
    compact_closed::cup_single,
    cospan::Cospan,
    cospan_algebra::{CospanAlgebra, NameAlgebra, PartitionAlgebra},
    frobenius::{FrobeniusMorphism, FrobeniusOperation},
};

type FM = FrobeniusMorphism<char, String>;

// ---------------------------------------------------------------------------
// PartitionAlgebra (Example 2.3)
// ---------------------------------------------------------------------------

#[test]
fn partition_unit_coherence() {
    let alg = PartitionAlgebra;
    let u: Cospan<char> = alg.unit();
    assert!(u.domain().is_empty());
    assert!(u.codomain().is_empty());
}

#[test]
fn partition_identity_is_identity_function() {
    let alg = PartitionAlgebra;
    // element: [] → [a, b] (a partition of {a, b} into one group)
    let element = Cospan::new(vec![], vec![0, 1], vec!['a', 'b']);
    let id = Cospan::<char>::identity(&vec!['a', 'b']);
    let mapped = alg.map_cospan(&id, &element).unwrap();
    // Identity should preserve the element's codomain
    assert_eq!(mapped.codomain(), element.codomain());
}

#[test]
fn partition_functoriality() {
    let alg = PartitionAlgebra;
    // element: [] → [a]
    let element = Cospan::new(vec![], vec![0], vec!['a']);
    // c1: [a] → [a] (identity)
    let c1 = Cospan::<char>::identity(&vec!['a']);
    // c2: [a] → [a] (identity)
    let c2 = Cospan::<char>::identity(&vec!['a']);
    // c1;c2
    let c12 = c1.compose(&c2).unwrap();

    // map_cospan(c1;c2, e) should equal map_cospan(c2, map_cospan(c1, e))
    let direct = alg.map_cospan(&c12, &element).unwrap();
    let step1 = alg.map_cospan(&c1, &element).unwrap();
    let sequential = alg.map_cospan(&c2, &step1).unwrap();
    assert_eq!(direct.codomain(), sequential.codomain());
}

#[test]
fn partition_lax_monoidal_coherence() {
    let alg = PartitionAlgebra;
    let a = Cospan::new(vec![], vec![0], vec!['a']);
    let b = Cospan::new(vec![], vec![0], vec!['b']);
    let combined: Cospan<char> = alg.lax_monoidal(&a, &b);
    assert!(combined.domain().is_empty());
    assert_eq!(combined.codomain(), vec!['a', 'b']);
}

#[test]
fn partition_merge_cospan() {
    let alg = PartitionAlgebra;
    // element: [] → [a, a] (two nodes, both type 'a')
    let element = Cospan::new(vec![], vec![0, 1], vec!['a', 'a']);
    // merge: [a, a] → [a] (both inputs map to same node)
    let merge = Cospan::new(vec![0, 0], vec![0], vec!['a']);
    let mapped = alg.map_cospan(&merge, &element).unwrap();
    assert!(mapped.domain().is_empty());
    assert_eq!(mapped.codomain(), vec!['a']);
}

#[test]
fn partition_split_cospan() {
    let alg = PartitionAlgebra;
    // element: [] → [a]
    let element = Cospan::new(vec![], vec![0], vec!['a']);
    // split: [a] → [a, a] (one input maps to one node, two outputs from same node)
    let split = Cospan::new(vec![0], vec![0, 0], vec!['a']);
    let mapped = alg.map_cospan(&split, &element).unwrap();
    assert!(mapped.domain().is_empty());
    assert_eq!(mapped.codomain(), vec!['a', 'a']);
}

// ---------------------------------------------------------------------------
// NameAlgebra (§4.1)
// ---------------------------------------------------------------------------

#[test]
fn name_unit_coherence() {
    let alg = NameAlgebra::<String>::new();
    let u: FM = alg.unit();
    assert!(u.domain().is_empty());
    assert!(u.codomain().is_empty());
}

#[test]
fn name_identity_is_identity_function() {
    let alg = NameAlgebra::<String>::new();
    let element: FM = FrobeniusOperation::Unit('a').into();
    let id = Cospan::<char>::identity(&vec!['a']);
    let mapped = alg.map_cospan(&id, &element).unwrap();
    assert_eq!(mapped.domain(), element.domain());
    assert_eq!(mapped.codomain(), element.codomain());
}

#[test]
fn name_functoriality() {
    let alg = NameAlgebra::<String>::new();
    let element: FM = FrobeniusOperation::Unit('a').into();
    let c1 = Cospan::<char>::identity(&vec!['a']);
    let c2 = Cospan::<char>::identity(&vec!['a']);
    let c12 = c1.compose(&c2).unwrap();

    let direct = alg.map_cospan(&c12, &element).unwrap();
    let step1 = alg.map_cospan(&c1, &element).unwrap();
    let sequential = alg.map_cospan(&c2, &step1).unwrap();
    assert_eq!(direct.domain(), sequential.domain());
    assert_eq!(direct.codomain(), sequential.codomain());
}

#[test]
fn name_lax_monoidal_coherence() {
    let alg = NameAlgebra::<String>::new();
    let a: FM = FrobeniusOperation::Unit('a').into();
    let b: FM = FrobeniusOperation::Unit('b').into();
    let combined = alg.lax_monoidal(&a, &b);
    assert!(combined.domain().is_empty());
    assert_eq!(combined.codomain(), vec!['a', 'b']);
}

#[test]
fn name_merge_cospan() {
    let alg = NameAlgebra::<String>::new();
    // element: [] → [a, a] (cup_single gives η;δ)
    let element: FM = cup_single('a');
    // merge: [a, a] → [a]
    let merge = Cospan::new(vec![0, 0], vec![0], vec!['a']);
    let mapped = alg.map_cospan(&merge, &element).unwrap();
    assert!(mapped.domain().is_empty());
    assert_eq!(mapped.codomain(), vec!['a']);
}

#[test]
fn name_split_cospan() {
    let alg = NameAlgebra::<String>::new();
    // element: [] → [a]
    let element: FM = FrobeniusOperation::Unit('a').into();
    // split: [a] → [a, a]
    let split = Cospan::new(vec![0], vec![0, 0], vec!['a']);
    let mapped = alg.map_cospan(&split, &element).unwrap();
    assert!(mapped.domain().is_empty());
    assert_eq!(mapped.codomain(), vec!['a', 'a']);
}

/// Verify that NameAlgebra is Send + Sync (important for async usage).
#[test]
fn name_algebra_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<NameAlgebra<String>>();
}
