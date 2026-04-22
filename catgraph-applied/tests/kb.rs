//! Tests for the congruence-closure engine in `prop::presentation::kb`.

use catgraph_applied::prop::presentation::kb::CongruenceClosure;
use catgraph_applied::prop::{Free, PropExpr, PropSignature};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum G {
    A,
    B,
    C,
}

impl PropSignature for G {
    fn source(&self) -> usize {
        1
    }
    fn target(&self) -> usize {
        1
    }
}

fn g(x: G) -> PropExpr<G> {
    Free::<G>::generator(x)
}

#[test]
fn cc_empty_is_reflexive() {
    let mut cc = CongruenceClosure::<G>::new(&[]);
    assert!(cc.are_equal(&g(G::A), &g(G::A)));
}

#[test]
fn cc_distinct_generators_not_equal() {
    let mut cc = CongruenceClosure::<G>::new(&[]);
    assert!(!cc.are_equal(&g(G::A), &g(G::B)));
}

#[test]
fn cc_seeded_equivalence_direct() {
    let mut cc = CongruenceClosure::<G>::new(&[(g(G::A), g(G::B))]);
    assert!(cc.are_equal(&g(G::A), &g(G::B)));
    assert!(cc.are_equal(&g(G::B), &g(G::A))); // symmetry
}

#[test]
fn cc_seeded_transitivity() {
    let mut cc = CongruenceClosure::<G>::new(&[
        (g(G::A), g(G::B)),
        (g(G::B), g(G::C)),
    ]);
    assert!(cc.are_equal(&g(G::A), &g(G::C))); // transitivity
}

#[test]
fn cc_congruence_through_compose() {
    // If A = B, then A ; A ~ B ; B via congruence.
    let mut cc = CongruenceClosure::<G>::new(&[(g(G::A), g(G::B))]);
    let aa = Free::<G>::compose(g(G::A), g(G::A)).unwrap();
    let bb = Free::<G>::compose(g(G::B), g(G::B)).unwrap();
    assert!(cc.are_equal(&aa, &bb));
}

#[test]
fn cc_congruence_through_tensor() {
    // If A = B, then A ⊗ C ~ B ⊗ C via congruence.
    let mut cc = CongruenceClosure::<G>::new(&[(g(G::A), g(G::B))]);
    let ac = Free::<G>::tensor(g(G::A), g(G::C));
    let bc = Free::<G>::tensor(g(G::B), g(G::C));
    assert!(cc.are_equal(&ac, &bc));
}

#[test]
fn cc_overlapping_equations_converge() {
    // The v0.5.0 killer case: overlapping scalar equations must join.
    // Simulate with: A ; A = B, A = C. Then A ; C = A ; A = B, so A ; C = B.
    let mut cc = CongruenceClosure::<G>::new(&[
        (Free::<G>::compose(g(G::A), g(G::A)).unwrap(), g(G::B)),
        (g(G::A), g(G::C)),
    ]);
    let ac = Free::<G>::compose(g(G::A), g(G::C)).unwrap();
    assert!(
        cc.are_equal(&ac, &g(G::B)),
        "overlapping equations should join under congruence closure"
    );
}

