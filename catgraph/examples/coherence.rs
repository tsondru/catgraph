//! CoherenceVerification and DifferentialCoherence API demonstration.
//!
//! Shows individual coherence checks (associator, unitors, braiding),
//! comprehensive verification via CoherenceVerification::verify_all,
//! and differential coherence analysis.

use catgraph::coherence::{
    verify_associator_coherence, verify_braiding_coherence, verify_left_unitor_coherence,
    verify_right_unitor_coherence, CoherenceVerification, DifferentialCoherence,
};
use catgraph::interval::{DiscreteInterval, ParallelIntervals};

/// Helper to build a ParallelIntervals from a list of (start, end) pairs.
fn make_parallel(intervals: Vec<(usize, usize)>) -> ParallelIntervals {
    let mut p = ParallelIntervals::new();
    for (s, e) in intervals {
        p.add_branch(DiscreteInterval::new(s, e));
    }
    p
}

// ============================================================================
// Individual Coherence Checks
// ============================================================================

fn individual_checks() {
    println!("=== Individual Coherence Checks ===\n");

    let x = make_parallel(vec![(0, 3)]);
    let y = make_parallel(vec![(5, 8)]);
    let z = make_parallel(vec![(10, 14)]);

    println!("x=[0,3], y=[5,8], z=[10,14]\n");

    // Associator: (x ⊗ y) ⊗ z ≅ x ⊗ (y ⊗ z)
    println!(
        "verify_associator_coherence(x, y, z) = {}",
        verify_associator_coherence(&x, &y, &z),
    );

    // Left unitor: I ⊗ x ≅ x
    println!(
        "verify_left_unitor_coherence(x)  = {}",
        verify_left_unitor_coherence(&x),
    );

    // Right unitor: x ⊗ I ≅ x
    println!(
        "verify_right_unitor_coherence(x) = {}",
        verify_right_unitor_coherence(&x),
    );

    // Braiding: x ⊗ y ≅ y ⊗ x
    println!(
        "verify_braiding_coherence(x, y)  = {}",
        verify_braiding_coherence(&x, &y),
    );
    println!();
}

// ============================================================================
// Comprehensive CoherenceVerification
// ============================================================================

fn comprehensive_verification() {
    println!("=== CoherenceVerification::verify_all ===\n");

    let intervals = vec![
        make_parallel(vec![(0, 2)]),
        make_parallel(vec![(2, 5)]),
        make_parallel(vec![(5, 10)]),
    ];

    let result = CoherenceVerification::verify_all(&intervals);
    println!("{result}");

    // Access individual fields
    println!("Fields:");
    println!("  associator_coherent  = {}", result.associator_coherent);
    println!("  left_unitor_coherent = {}", result.left_unitor_coherent);
    println!("  right_unitor_coherent = {}", result.right_unitor_coherent);
    println!("  braiding_coherent    = {}", result.braiding_coherent);
    println!("  associator_tests     = {} (3^3 = 27)", result.associator_tests);
    println!("  braiding_tests       = {} (3^2 = 9)", result.braiding_tests);
    println!("  fully_coherent       = {}", result.fully_coherent);

    // Empty case
    let empty_result = CoherenceVerification::verify_all(&[]);
    println!("\nEmpty interval set:");
    println!("  fully_coherent = {}, tests = {}", empty_result.fully_coherent, empty_result.associator_tests);
    println!();
}

// ============================================================================
// DifferentialCoherence
// ============================================================================

fn differential_coherence() {
    println!("=== DifferentialCoherence ===\n");

    let intervals = vec![
        make_parallel(vec![(0, 2)]),
        make_parallel(vec![(2, 5)]),
    ];

    let result = DifferentialCoherence::verify(&intervals);
    println!("{result}");

    // Key methods
    println!("has_categorical_curvature() = {}", result.has_categorical_curvature());
    println!("coherence_defect()          = {:.6}", result.coherence_defect());

    // Access fields
    println!("\nFields:");
    println!("  differential_coherent = {}", result.differential_coherent);
    println!("  coherence_form_closed = {}", result.coherence_form_closed);
    println!("  conservation_ratio    = {:.4}", result.conservation_ratio);
    println!("  non_closure_count     = {}", result.non_closure_count);

    // Algebraic sub-result
    println!(
        "\nAlgebraic sub-result: fully_coherent = {}",
        result.algebraic.fully_coherent,
    );
    println!();
}

// ============================================================================
// Multi-Branch Coherence
// ============================================================================

fn multi_branch() {
    println!("=== Multi-Branch Coherence ===\n");

    // Two multi-branch parallel intervals
    let intervals = vec![
        make_parallel(vec![(0, 2), (3, 5)]),
        make_parallel(vec![(10, 15)]),
    ];

    let result = DifferentialCoherence::verify(&intervals);
    println!("Multi-branch intervals:");
    println!("  differential_coherent = {}", result.differential_coherent);
    println!("  curvature             = {}", if result.has_categorical_curvature() { "present" } else { "flat" });
    println!("  conservation_ratio    = {:.4}", result.conservation_ratio);
    println!();
}

fn main() {
    individual_checks();
    comprehensive_verification();
    differential_coherence();
    multi_branch();
}
