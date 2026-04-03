//! TemporalComplex, ConservationResult, and StokesError API demonstration.
//!
//! Shows temporal complex construction from intervals, conservation verification,
//! differential forms, integration, cospan chain generation and composition,
//! and error handling.

use catgraph::category::Composable;
use catgraph::interval::DiscreteInterval;
use catgraph::stokes::{ConservationResult, StokesError, TemporalComplex};

// ============================================================================
// Construction and Basic Properties
// ============================================================================

fn construction() {
    println!("=== Temporal Complex Construction ===\n");

    let intervals = vec![
        DiscreteInterval::new(0, 2),
        DiscreteInterval::new(2, 5),
        DiscreteInterval::new(5, 7),
    ];

    let complex = TemporalComplex::from_intervals(&intervals).unwrap();
    println!("Intervals: [0,2], [2,5], [5,7]");
    println!("num_time_steps = {} (vertices)", complex.num_time_steps());
    println!("num_intervals  = {} (edges)", complex.num_intervals());
    println!("time_points    = {:?}", complex.time_points());
    println!("step_counts    = {:?}", complex.step_counts());
    println!();
}

// ============================================================================
// Differential Forms and Integration
// ============================================================================

fn forms_and_integration() {
    println!("=== Differential Forms & Integration ===\n");

    let intervals = vec![
        DiscreteInterval::new(0, 3),
        DiscreteInterval::new(3, 7),
        DiscreteInterval::new(7, 10),
    ];

    let complex = TemporalComplex::from_intervals(&intervals).unwrap();

    // 1-form: step counts as coefficients
    let form = complex.intervals_to_form();
    println!("intervals_to_form = {:?}", form);

    // Exterior derivative: always zero for 1D complexes
    let d_form = complex.exterior_derivative(&form);
    println!("exterior_derivative = {:?}  (trivially zero in dim-1)", d_form);

    // Integration: sum of coefficients
    let integral = complex.integrate(&form);
    println!("integrate(form) = {integral}  (total span = 10 - 0 = 10)");
    println!();
}

// ============================================================================
// Conservation Verification
// ============================================================================

fn conservation() {
    println!("=== Conservation Verification ===\n");

    // Contiguous, monotonic sequence
    let intervals = vec![
        DiscreteInterval::new(0, 1),
        DiscreteInterval::new(1, 3),
        DiscreteInterval::new(3, 6),
    ];
    let complex = TemporalComplex::from_intervals(&intervals).unwrap();
    let result = complex.verify_conservation();

    print_conservation(&result, "Contiguous [0,1],[1,3],[3,6]");
    println!("average_complexity = {:.4}", result.average_complexity());
    println!("is_well_formed     = {}", result.is_well_formed());

    // Single interval
    println!();
    let single = TemporalComplex::from_intervals(&[DiscreteInterval::new(5, 12)]).unwrap();
    let single_result = single.verify_conservation();
    print_conservation(&single_result, "Single [5,12]");
    println!();
}

fn print_conservation(r: &ConservationResult, label: &str) {
    println!("{label}:");
    println!("  is_conserved      = {}", r.is_conserved);
    println!("  is_closed         = {}", r.is_closed);
    println!("  is_contiguous     = {}", r.is_contiguous);
    println!("  is_monotonic      = {}", r.is_monotonic);
    println!("  total_complexity  = {}", r.total_complexity);
    println!("  num_intervals     = {}", r.num_intervals);
    println!("  num_time_steps    = {}", r.num_time_steps);
}

// ============================================================================
// Cospan Chain
// ============================================================================

fn cospan_chain() {
    println!("=== Cospan Chain ===\n");

    let intervals = vec![
        DiscreteInterval::new(0, 2),
        DiscreteInterval::new(2, 5),
        DiscreteInterval::new(5, 7),
    ];
    let complex = TemporalComplex::from_intervals(&intervals).unwrap();

    // Generate individual cospans
    let chain = complex.to_cospan_chain();
    println!("Chain length: {}", chain.len());
    for (i, c) in chain.iter().enumerate() {
        println!(
            "  cospan[{i}]: left={:?}, right={:?}, middle={:?}",
            c.left_to_middle(),
            c.right_to_middle(),
            c.middle(),
        );
    }

    // Verify composability
    println!("\nAdjacent composability:");
    for i in 0..chain.len() - 1 {
        let codomain_len = chain[i].codomain().len();
        let domain_len = chain[i + 1].domain().len();
        println!(
            "  cospan[{i}] codomain({codomain_len}) -> cospan[{}] domain({domain_len}): composable",
            i + 1,
        );
    }

    // Compose full chain
    let composite = complex.compose_cospan_chain().unwrap();
    println!("\nComposite cospan:");
    println!("  domain   = {:?}", composite.domain());
    println!("  codomain = {:?}", composite.codomain());
    println!("  middle   = {:?}", composite.middle());
    println!();
}

// ============================================================================
// Error Handling
// ============================================================================

fn error_handling() {
    println!("=== StokesError Handling ===\n");

    // Empty intervals
    let empty = TemporalComplex::from_intervals(&[]);
    match empty {
        Err(StokesError::EmptyIntervals) => println!("EmptyIntervals: {}", StokesError::EmptyIntervals),
        other => println!("Unexpected: {other:?}"),
    }

    // Singleton interval produces InsufficientPoints if deduplication collapses to 1 point
    let singleton = TemporalComplex::from_intervals(&[DiscreteInterval::singleton(5)]);
    match singleton {
        Err(StokesError::InsufficientPoints(n)) => {
            println!("InsufficientPoints({}): {}", n, StokesError::InsufficientPoints(n));
        }
        Ok(_) => println!("Singleton created successfully (2+ distinct points)"),
        Err(e) => println!("Other error: {e}"),
    }
    println!();
}

fn main() {
    construction();
    forms_and_integration();
    conservation();
    cospan_chain();
    error_handling();
}
