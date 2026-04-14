//! Integration tests for PetriNet: chemical reactions, reachability, composition, cospan roundtrip.

use std::collections::HashMap;
use catgraph::cospan::Cospan;
use catgraph_applied::petri_net::{Marking, PetriNet, Transition};
use rust_decimal::Decimal;

/// Shorthand for `Decimal::from(n)`.
fn d(n: i64) -> Decimal {
    Decimal::from(n)
}

// ---------------------------------------------------------------------------
// Chemical reactions
// ---------------------------------------------------------------------------

#[test]
fn combustion_h2_o2_h2o() {
    let net: PetriNet<&str> = PetriNet::new(
        vec!["H2", "O2", "H2O"],
        vec![Transition::new(vec![(0, d(2)), (1, d(1))], vec![(2, d(2))])],
    );
    let m0 = Marking::from_vec(vec![(0, d(4)), (1, d(2))]);
    let m1 = net.fire(0, &m0).unwrap();
    let m2 = net.fire(0, &m1).unwrap();
    assert_eq!(m2.get(0), Decimal::ZERO);
    assert_eq!(m2.get(1), Decimal::ZERO);
    assert_eq!(m2.get(2), d(4));
    assert!(net.enabled(&m2).is_empty());
}

#[test]
fn two_step_synthesis() {
    let net: PetriNet<&str> = PetriNet::new(
        vec!["A", "B", "C", "D", "E"],
        vec![
            Transition::new(vec![(0, d(1)), (1, d(1))], vec![(2, d(1))]),
            Transition::new(vec![(2, d(1)), (3, d(1))], vec![(4, d(1))]),
        ],
    );
    let m0 = Marking::from_vec(vec![(0, d(1)), (1, d(1)), (3, d(1))]);
    assert_eq!(net.enabled(&m0), vec![0]);
    let m1 = net.fire(0, &m0).unwrap();
    assert_eq!(m1.get(2), d(1));
    assert_eq!(net.enabled(&m1), vec![1]);
    let m2 = net.fire(1, &m1).unwrap();
    assert_eq!(m2.get(4), d(1));
}

#[test]
fn haber_process_stoichiometry() {
    let net: PetriNet<&str> = PetriNet::new(
        vec!["N2", "H2", "NH3"],
        vec![Transition::new(vec![(0, d(1)), (1, d(3))], vec![(2, d(2))])],
    );
    let m0 = Marking::from_vec(vec![(0, d(1)), (1, d(3))]);
    let m1 = net.fire(0, &m0).unwrap();
    assert_eq!(m1.get(2), d(2));
    assert!(net.enabled(&m1).is_empty());
}

// ---------------------------------------------------------------------------
// Reachability
// ---------------------------------------------------------------------------

#[test]
fn producer_consumer_bounded_buffer() {
    let net: PetriNet<&str> = PetriNet::new(
        vec!["empty", "full"],
        vec![
            Transition::new(vec![(0, d(1))], vec![(1, d(1))]),
            Transition::new(vec![(1, d(1))], vec![(0, d(1))]),
        ],
    );
    let m0 = Marking::from_vec(vec![(0, d(3))]);
    let reachable = net.reachable(&m0, 10);
    assert_eq!(reachable.len(), 4);
    assert!(net.can_reach(&m0, &Marking::from_vec(vec![(1, d(3))]), 10));
    assert!(!net.can_reach(&m0, &Marking::from_vec(vec![(0, d(4))]), 10));
}

#[test]
fn deadlock_detection() {
    let net: PetriNet<&str> = PetriNet::new(
        vec!["fork0", "fork1", "think0", "think1", "eat0", "eat1"],
        vec![
            Transition::new(vec![(2, d(1)), (0, d(1)), (1, d(1))], vec![(4, d(1))]),
            Transition::new(vec![(4, d(1))], vec![(2, d(1)), (0, d(1)), (1, d(1))]),
            Transition::new(vec![(3, d(1)), (0, d(1)), (1, d(1))], vec![(5, d(1))]),
            Transition::new(vec![(5, d(1))], vec![(3, d(1)), (0, d(1)), (1, d(1))]),
        ],
    );
    let m0 = Marking::from_vec(vec![(0, d(1)), (1, d(1)), (2, d(1)), (3, d(1))]);
    let eating0 = Marking::from_vec(vec![(3, d(1)), (4, d(1))]);
    assert!(net.can_reach(&m0, &eating0, 5));
}

// ---------------------------------------------------------------------------
// Composition
// ---------------------------------------------------------------------------

#[test]
fn sequential_pipeline() {
    let step1: PetriNet<char> = PetriNet::new(
        vec!['A', 'B'],
        vec![Transition::new(vec![(0, d(1))], vec![(1, d(1))])],
    );
    let step2: PetriNet<char> = PetriNet::new(
        vec!['B', 'C'],
        vec![Transition::new(vec![(0, d(1))], vec![(1, d(1))])],
    );
    let pipeline = step1.sequential(&step2).unwrap();
    assert_eq!(pipeline.place_count(), 3);
    let m0 = Marking::from_vec(vec![(0, d(1))]);
    let target = Marking::from_vec(vec![(2, d(1))]);
    assert!(pipeline.can_reach(&m0, &target, 5));
}

#[test]
fn parallel_independence() {
    let a: PetriNet<char> = PetriNet::new(
        vec!['a', 'b'],
        vec![Transition::new(vec![(0, d(1))], vec![(1, d(1))])],
    );
    let b: PetriNet<char> = PetriNet::new(
        vec!['x', 'y'],
        vec![Transition::new(vec![(0, d(1))], vec![(1, d(1))])],
    );
    let combined = a.parallel(&b);
    let m0 = Marking::from_vec(vec![(0, d(1))]);
    let m1 = combined.fire(0, &m0).unwrap();
    assert_eq!(m1.get(1), d(1));
    assert_eq!(m1.get(2), Decimal::ZERO);
}

// ---------------------------------------------------------------------------
// Cospan roundtrip
// ---------------------------------------------------------------------------

#[test]
fn cospan_roundtrip_preserves_structure() {
    let cospan: Cospan<char> = Cospan::new(vec![0, 1, 1, 1], vec![2, 2], vec!['N', 'H', 'A']);
    let net = PetriNet::from_cospan(&cospan);
    let back = net.transition_as_cospan(0);
    assert_eq!(back.middle(), cospan.middle());
    let mut left_counts_orig: HashMap<usize, usize> = HashMap::new();
    for &idx in cospan.left_to_middle() { *left_counts_orig.entry(idx).or_insert(0) += 1; }
    let mut left_counts_back: HashMap<usize, usize> = HashMap::new();
    for &idx in back.left_to_middle() { *left_counts_back.entry(idx).or_insert(0) += 1; }
    assert_eq!(left_counts_orig, left_counts_back);
}
