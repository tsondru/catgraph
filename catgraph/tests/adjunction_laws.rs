//! Integration tests for the adjunction module.
//!
//! Defines a concrete `SimpleAdjunction` implementing `ZPrimeOps`, then
//! verifies triangle identities, verification sequence counts, and
//! irreducibility indicators.

use catgraph::adjunction::{AdjunctionIrreducibility, AdjunctionVerification, ZPrimeOps};
use catgraph::computation_state::ComputationState;
use catgraph::interval::DiscreteInterval;

// ---------------------------------------------------------------------------
// Test fixture
// ---------------------------------------------------------------------------

/// A well-formed adjunction: Z' maps state to its natural interval,
/// Z maps interval back to a state. Round-trip is exact for states
/// with nonzero complexity.
struct SimpleAdjunction;

impl ZPrimeOps for SimpleAdjunction {
    fn zprime(state: &ComputationState) -> DiscreteInterval {
        state.to_interval()
    }

    fn z(interval: &DiscreteInterval) -> ComputationState {
        ComputationState::new(interval.start, interval.end - interval.start)
    }

    fn unit_at(state: &ComputationState) -> ComputationState {
        let interval = Self::zprime(state);
        Self::z(&interval)
    }

    fn counit_at(interval: &DiscreteInterval) -> DiscreteInterval {
        let state = Self::z(interval);
        Self::zprime(&state)
    }

    fn verify_triangle_1(state: &ComputationState) -> bool {
        let zp_c = Self::zprime(state);
        let eta_c = Self::unit_at(state);
        let zp_eta_c = Self::zprime(&eta_c);
        let epsilon_zp_c = Self::counit_at(&zp_c);
        epsilon_zp_c == zp_c && zp_eta_c == zp_c
    }

    fn verify_triangle_2(interval: &DiscreteInterval) -> bool {
        let z_i = Self::z(interval);
        let epsilon_i = Self::counit_at(interval);
        let z_epsilon_i = Self::z(&epsilon_i);
        let eta_z_i = Self::unit_at(&z_i);
        z_epsilon_i == z_i && eta_z_i == z_i
    }
}

impl AdjunctionIrreducibility for SimpleAdjunction {}

/// Helper: build a walk of `n` states starting from `ComputationState::initial().next()`.
fn make_walk(n: usize) -> Vec<ComputationState> {
    (0..n)
        .scan(ComputationState::initial(), |s, _| {
            *s = s.next();
            Some(s.clone())
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Triangle identities
// ---------------------------------------------------------------------------

#[test]
fn triangle_identities_hold() {
    let states = make_walk(6);

    for s in &states {
        assert!(
            SimpleAdjunction::verify_triangle_1(s),
            "triangle 1 failed at step {}",
            s.step
        );
        let interval = SimpleAdjunction::zprime(s);
        assert!(
            SimpleAdjunction::verify_triangle_2(&interval),
            "triangle 2 failed at interval [{}, {}]",
            interval.start,
            interval.end
        );
    }
}

// ---------------------------------------------------------------------------
// Verification sequence
// ---------------------------------------------------------------------------

#[test]
fn verify_sequence_counts_correctly() {
    let states = make_walk(5);
    let result = AdjunctionVerification::verify_sequence::<SimpleAdjunction>(&states);

    assert!(result.triangle_identities_hold);
    assert!(result.is_adjoint_pair);
    assert_eq!(result.triangle_1_results.len(), 5);
    assert_eq!(result.triangle_2_results.len(), 5);
    assert_eq!(result.triangle_1_failures(), 0);
    assert_eq!(result.triangle_2_failures(), 0);
}

// ---------------------------------------------------------------------------
// Adjunction gap
// ---------------------------------------------------------------------------

#[test]
fn adjunction_gap_zero_for_reducible() {
    // States with nonzero complexity have exact Z(Z'(c)) == c round-trip,
    // so gap should be zero.
    let state = ComputationState::new(3, 5);
    let gap = SimpleAdjunction::adjunction_gap(&state);
    assert!(
        gap.abs() < 1e-10,
        "expected zero gap for exact round-trip, got {gap}"
    );
}

#[test]
fn adjunction_gap_positive_for_irreducible() {
    // The zero-complexity state maps to a 1-step interval (min clamp),
    // so Z(Z'(state)) has complexity 1 != 0. Gap should be positive.
    let state = ComputationState::new(0, 0);
    let gap = SimpleAdjunction::adjunction_gap(&state);
    assert!(
        gap > 0.0,
        "expected positive gap for lossy round-trip, got {gap}"
    );
}

// ---------------------------------------------------------------------------
// Irreducibility indicator
// ---------------------------------------------------------------------------

#[test]
fn irreducibility_indicator_averages() {
    let states = make_walk(5);
    let indicator = SimpleAdjunction::adjunction_irreducibility_indicator(&states);

    // All states in the walk have complexity >= 1, so round-trip is exact
    // and each gap is 0. Indicator should be 0.
    assert!(
        indicator.abs() < 1e-10,
        "expected zero indicator for exact walk, got {indicator}"
    );

    // Empty sequence returns 0.
    let empty = SimpleAdjunction::adjunction_irreducibility_indicator(&[]);
    assert!((empty - 0.0).abs() < 1e-10);
}
