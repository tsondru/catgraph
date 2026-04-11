//! Integration tests for the complexity module.
//!
//! Verifies algebraic properties of `StepCount`: sequential composition is
//! associative with additive identity, parallel composition uses max, and
//! ordering is consistent.

use catgraph::complexity::{Complexity, StepCount};

// ---------------------------------------------------------------------------
// Sequential composition
// ---------------------------------------------------------------------------

#[test]
fn sequential_composition_associative() {
    let a = StepCount::new(3);
    let b = StepCount::new(5);
    let c = StepCount::new(7);

    let left = a.sequential(&b).sequential(&c);
    let right = a.sequential(&b.sequential(&c));

    assert_eq!(left, right);
    assert_eq!(left, StepCount::new(15));
}

#[test]
fn zero_is_additive_identity() {
    let a = StepCount::new(42);
    let zero = StepCount::zero();

    assert_eq!(a.sequential(&zero), a);
    assert_eq!(zero.sequential(&a), a);
}

// ---------------------------------------------------------------------------
// Parallel composition
// ---------------------------------------------------------------------------

#[test]
fn parallel_uses_max() {
    let a = StepCount::new(3);
    let b = StepCount::new(7);

    assert_eq!(a.parallel(&b), StepCount::new(7));
    assert_eq!(b.parallel(&a), StepCount::new(7));
    assert_eq!(a.parallel(&a), a);
}

// ---------------------------------------------------------------------------
// Accessors and predicates
// ---------------------------------------------------------------------------

#[test]
fn as_steps_roundtrip() {
    for n in [0, 1, 42, 1000] {
        assert_eq!(StepCount::new(n).as_steps(), n);
    }
}

#[test]
fn is_zero_correct() {
    assert!(StepCount::zero().is_zero());
    assert!(!StepCount::one().is_zero());
    assert!(!StepCount::new(5).is_zero());
}

// ---------------------------------------------------------------------------
// Ordering
// ---------------------------------------------------------------------------

#[test]
fn ordering_consistent() {
    assert!(StepCount::new(3) < StepCount::new(5));
    assert!(StepCount::new(5) > StepCount::new(3));
    assert!(StepCount::new(5) == StepCount::new(5));
}
