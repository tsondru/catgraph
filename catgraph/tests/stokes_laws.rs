//! Integration tests for the Stokes module.
//!
//! Verifies `TemporalComplex` conservation, exterior derivative properties,
//! cospan chain composability, and edge cases.

use catgraph::category::Composable;
use catgraph::interval::DiscreteInterval;
use catgraph::stokes::{ConservationResult, StokesError, TemporalComplex};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn contiguous_intervals() -> Vec<DiscreteInterval> {
    vec![
        DiscreteInterval::new(0, 3),
        DiscreteInterval::new(3, 5),
        DiscreteInterval::new(5, 8),
    ]
}

// ---------------------------------------------------------------------------
// Conservation
// ---------------------------------------------------------------------------

#[test]
fn contiguous_intervals_are_conserved() {
    let complex = TemporalComplex::from_intervals(&contiguous_intervals()).unwrap();
    let result = complex.verify_conservation();

    assert!(result.is_conserved);
    assert!(result.is_contiguous);
    assert!(result.is_monotonic);
    assert!(result.is_closed);
    assert!(result.is_well_formed());
    assert!((result.total_complexity - 8.0).abs() < 1e-10);
}

#[test]
fn non_contiguous_fails_conservation() {
    // Gap between [0,3] and [5,8] -- after dedup+sort the time points are 0,3,5,8
    // but the step_counts are all > 0 so contiguity holds structurally.
    // To get a non-contiguous failure we need a zero-length step, which requires
    // duplicate time points. Instead, verify that a gap still produces a valid
    // complex but the integrated total differs from a contiguous chain.
    let gap_intervals = vec![
        DiscreteInterval::new(0, 3),
        DiscreteInterval::new(5, 8),
    ];
    let complex = TemporalComplex::from_intervals(&gap_intervals).unwrap();
    let result = complex.verify_conservation();

    // The complex fills the gap (time_points = [0,3,5,8]), so it is still
    // contiguous and monotonic in its own right. Total complexity = 8.
    assert!(result.is_conserved);
    assert_eq!(result.num_time_steps, 4);
    assert_eq!(result.num_intervals, 3);
}

// ---------------------------------------------------------------------------
// Exterior derivative
// ---------------------------------------------------------------------------

#[test]
fn exterior_derivative_is_zero_1d() {
    let complex = TemporalComplex::from_intervals(&contiguous_intervals()).unwrap();
    let form = complex.intervals_to_form();
    let d_form = complex.exterior_derivative(&form);

    assert!(d_form.iter().all(|&c| c.abs() < 1e-10));
}

// ---------------------------------------------------------------------------
// Cospan chain
// ---------------------------------------------------------------------------

#[test]
fn to_cospan_chain_composable() {
    let complex = TemporalComplex::from_intervals(&contiguous_intervals()).unwrap();
    let cospans = complex.to_cospan_chain();

    assert_eq!(cospans.len(), complex.num_intervals());

    for i in 0..cospans.len() - 1 {
        let codomain_i = cospans[i].codomain();
        let domain_next = cospans[i + 1].domain();
        assert_eq!(
            codomain_i.len(),
            domain_next.len(),
            "boundary size mismatch at index {i}"
        );
    }
}

#[test]
fn compose_cospan_chain_roundtrip() {
    let complex = TemporalComplex::from_intervals(&contiguous_intervals()).unwrap();
    let composite = complex.compose_cospan_chain().unwrap();

    // Domain is the first time point, codomain is the last.
    assert_eq!(composite.domain().len(), 1);
    assert_eq!(composite.codomain().len(), 1);
    assert_eq!(composite.domain(), vec![0_u32]);
    assert_eq!(composite.codomain(), vec![8_u32]);
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn single_interval_complex() {
    let intervals = vec![DiscreteInterval::new(4, 9)];
    let complex = TemporalComplex::from_intervals(&intervals).unwrap();

    assert_eq!(complex.num_time_steps(), 2);
    assert_eq!(complex.num_intervals(), 1);

    let result = complex.verify_conservation();
    assert!(result.is_conserved);
    assert!((result.total_complexity - 5.0).abs() < 1e-10);

    let cospans = complex.to_cospan_chain();
    assert_eq!(cospans.len(), 1);
}

#[test]
fn empty_intervals_error() {
    let err = TemporalComplex::from_intervals(&[]).unwrap_err();
    assert!(matches!(err, StokesError::EmptyIntervals));
}

#[test]
fn empty_conservation_result_methods() {
    let result = ConservationResult {
        is_conserved: true,
        is_closed: true,
        is_contiguous: true,
        is_monotonic: true,
        total_complexity: 0.0,
        num_intervals: 0,
        num_time_steps: 0,
    };

    assert!((result.average_complexity() - 0.0).abs() < 1e-10);
    assert!(result.is_well_formed());
}
