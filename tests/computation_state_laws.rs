//! Integration tests for the `computation_state` module.
//!
//! Verifies `ComputationState` construction, stepping, interval mapping,
//! fingerprint-based cycle detection, and multi-step walks.

use catgraph::computation_state::ComputationState;

// ---------------------------------------------------------------------------
// Construction and initial state
// ---------------------------------------------------------------------------

#[test]
fn initial_state_zeroed() {
    let s = ComputationState::initial();

    assert_eq!(s.step, 0);
    assert_eq!(s.complexity, 0);
    assert_eq!(s.fingerprint, None);
}

// ---------------------------------------------------------------------------
// Stepping
// ---------------------------------------------------------------------------

#[test]
fn next_increments_both() {
    let s0 = ComputationState::new(2, 3);
    let s1 = s0.next();

    assert_eq!(s1.step, 3);
    assert_eq!(s1.complexity, 4);
    // next() drops fingerprint.
    assert_eq!(s1.fingerprint, None);
}

// ---------------------------------------------------------------------------
// Interval mapping
// ---------------------------------------------------------------------------

#[test]
fn to_interval_mapping() {
    let s = ComputationState::new(2, 5);
    let interval = s.to_interval();

    assert_eq!(interval.start, 2);
    assert_eq!(interval.end, 7); // 2 + max(5, 1) = 7
}

#[test]
fn identity_interval_when_zero_complexity() {
    // When complexity = 0, interval is [step, step + 1] (1-step minimum).
    let s = ComputationState::new(4, 0);
    let interval = s.to_interval();

    assert_eq!(interval.start, 4);
    assert_eq!(interval.end, 5);
    assert_eq!(interval.steps(), 1);
}

#[test]
fn to_interval_with_equal_step_and_complexity() {
    // When step == complexity (and > 0), interval is [n, 2n].
    let s = ComputationState::new(3, 3);
    let interval = s.to_interval();

    assert_eq!(interval.start, 3);
    assert_eq!(interval.end, 6); // 3 + max(3, 1) = 6
}

// ---------------------------------------------------------------------------
// Fingerprint and cycle detection
// ---------------------------------------------------------------------------

#[test]
fn fingerprint_enables_cycle_detection() {
    let a = ComputationState::with_fingerprint(1, 2, 0xABCD);
    let b = ComputationState::with_fingerprint(3, 4, 0xABCD);
    let c = ComputationState::with_fingerprint(1, 2, 0x1234);

    // Same fingerprint does not imply equality (different step/complexity).
    assert_ne!(a, b);
    assert_eq!(a.fingerprint, b.fingerprint);

    // Same step/complexity but different fingerprint are not equal.
    assert_ne!(a, c);
}

// ---------------------------------------------------------------------------
// Multi-step walk
// ---------------------------------------------------------------------------

#[test]
fn multi_step_walk() {
    let mut state = ComputationState::initial();
    let mut prev_step = state.step;

    for _ in 0..10 {
        state = state.next();
        assert!(
            state.step > prev_step,
            "step must be strictly increasing"
        );
        assert_eq!(state.step, state.complexity, "next() keeps step == complexity");
        prev_step = state.step;
    }

    assert_eq!(state.step, 10);
}
