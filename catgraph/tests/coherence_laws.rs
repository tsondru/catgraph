//! Integration tests for the coherence module.
//!
//! Verifies symmetric monoidal coherence conditions (associator, unitors,
//! braiding) and differential coherence interpretation for `ParallelIntervals`.

use catgraph::coherence::{
    verify_associator_coherence, verify_braiding_coherence, verify_left_unitor_coherence,
    verify_right_unitor_coherence, CoherenceVerification, DifferentialCoherence,
};
use catgraph::interval::{DiscreteInterval, ParallelIntervals};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_parallel(intervals: &[(usize, usize)]) -> ParallelIntervals {
    let mut p = ParallelIntervals::new();
    for &(s, e) in intervals {
        p.add_branch(DiscreteInterval::new(s, e));
    }
    p
}

// ---------------------------------------------------------------------------
// Individual coherence checks
// ---------------------------------------------------------------------------

#[test]
fn associator_coherence_passes() {
    let a = make_parallel(&[(0, 3)]);
    let b = make_parallel(&[(3, 7)]);
    let c = make_parallel(&[(7, 12)]);

    assert!(verify_associator_coherence(&a, &b, &c));
}

#[test]
fn unitor_coherence_passes() {
    let x = make_parallel(&[(0, 5), (10, 15)]);

    assert!(verify_left_unitor_coherence(&x));
    assert!(verify_right_unitor_coherence(&x));
}

#[test]
fn braiding_coherence_passes() {
    let x = make_parallel(&[(0, 3)]);
    let y = make_parallel(&[(5, 9)]);

    assert!(verify_braiding_coherence(&x, &y));
}

// ---------------------------------------------------------------------------
// Comprehensive verification
// ---------------------------------------------------------------------------

#[test]
fn verify_all_aggregates_correctly() {
    let intervals = vec![
        make_parallel(&[(0, 2)]),
        make_parallel(&[(2, 5)]),
        make_parallel(&[(5, 10)]),
    ];
    let result = CoherenceVerification::verify_all(&intervals);

    assert!(result.fully_coherent);
    assert!(result.associator_coherent);
    assert!(result.left_unitor_coherent);
    assert!(result.right_unitor_coherent);
    assert!(result.braiding_coherent);
    // 3 elements: 3^3 = 27 associator tests, 3^2 = 9 braiding tests.
    assert_eq!(result.associator_tests, 27);
    assert_eq!(result.braiding_tests, 9);
}

#[test]
fn verify_all_empty_collection() {
    let result = CoherenceVerification::verify_all(&[]);

    assert!(result.fully_coherent);
    assert_eq!(result.associator_tests, 0);
    assert_eq!(result.braiding_tests, 0);
}

// ---------------------------------------------------------------------------
// Differential coherence
// ---------------------------------------------------------------------------

#[test]
fn differential_coherence_zero_curvature() {
    let intervals = vec![
        make_parallel(&[(0, 2)]),
        make_parallel(&[(2, 5)]),
    ];
    let dc = DifferentialCoherence::verify(&intervals);

    assert!(dc.differential_coherent);
    assert!(dc.coherence_form_closed);
    assert!(!dc.has_categorical_curvature());
    assert!(dc.coherence_defect() < 1e-10);
    assert_eq!(dc.non_closure_count, 0);
    assert!((dc.conservation_ratio - 1.0).abs() < 1e-10);
}

#[test]
fn differential_coherence_empty_is_flat() {
    let dc = DifferentialCoherence::verify(&[]);

    assert!(dc.differential_coherent);
    assert!(!dc.has_categorical_curvature());
    assert!(dc.coherence_defect() < 1e-10);
}
