//! ZPrimeOps, AdjunctionVerification, and AdjunctionIrreducibility API demonstration.
//!
//! Defines a concrete SimpleAdjunction implementing the Z' ⊣ Z adjunction,
//! then verifies triangle identities and computes irreducibility indicators.

use catgraph::adjunction::{
    AdjunctionIrreducibility, AdjunctionVerification, ZPrimeOps,
};
use catgraph::computation_state::ComputationState;
use catgraph::interval::DiscreteInterval;

// ============================================================================
// Concrete Implementation
// ============================================================================

/// A simple adjunction where:
/// - Z': state -> its natural interval [step, step + max(complexity,1)]
/// - Z: interval -> state with step=start, complexity=end-start
struct SimpleAdjunction;

impl ZPrimeOps for SimpleAdjunction {
    fn zprime(state: &ComputationState) -> DiscreteInterval {
        state.to_interval()
    }

    fn z(interval: &DiscreteInterval) -> ComputationState {
        ComputationState::new(interval.start, interval.end - interval.start)
    }

    fn unit_at(state: &ComputationState) -> ComputationState {
        // eta_c = Z(Z'(c))
        let interval = Self::zprime(state);
        Self::z(&interval)
    }

    fn counit_at(interval: &DiscreteInterval) -> DiscreteInterval {
        // epsilon_i = Z'(Z(i))
        let state = Self::z(interval);
        Self::zprime(&state)
    }

    fn verify_triangle_1(state: &ComputationState) -> bool {
        // epsilon_{Z'(c)} . Z'(eta_c) = id_{Z'(c)}
        let zp_c = Self::zprime(state);
        let eta_c = Self::unit_at(state);
        let zp_eta_c = Self::zprime(&eta_c);
        let epsilon_zp_c = Self::counit_at(&zp_c);
        epsilon_zp_c == zp_c && zp_eta_c == zp_c
    }

    fn verify_triangle_2(interval: &DiscreteInterval) -> bool {
        // Z(epsilon_i) . eta_{Z(i)} = id_{Z(i)}
        let z_i = Self::z(interval);
        let epsilon_i = Self::counit_at(interval);
        let z_epsilon_i = Self::z(&epsilon_i);
        let eta_z_i = Self::unit_at(&z_i);
        z_epsilon_i == z_i && eta_z_i == z_i
    }
}

impl AdjunctionIrreducibility for SimpleAdjunction {}

// ============================================================================
// Z' and Z Functors
// ============================================================================

fn functor_demo() {
    println!("=== Z' and Z Functors ===\n");

    let state = ComputationState::new(2, 5);
    let interval = SimpleAdjunction::zprime(&state);
    println!("Z'(step=2, complexity=5) = {interval}");

    let roundtrip = SimpleAdjunction::z(&interval);
    println!("Z({interval}) = step={}, complexity={}", roundtrip.step, roundtrip.complexity);
    println!("Roundtrip Z(Z'(c)) == c? {}", roundtrip == state);

    // Zero complexity state
    let zero_state = ComputationState::new(3, 0);
    let zero_interval = SimpleAdjunction::zprime(&zero_state);
    println!("\nZ'(step=3, complexity=0) = {zero_interval}  (min 1-step)");
    let back = SimpleAdjunction::z(&zero_interval);
    println!("Z({zero_interval}) = step={}, complexity={}", back.step, back.complexity);
    println!();
}

// ============================================================================
// Unit and Counit
// ============================================================================

fn unit_counit() {
    println!("=== Unit (eta) and Counit (epsilon) ===\n");

    let state = ComputationState::new(1, 4);
    let eta = SimpleAdjunction::unit_at(&state);
    println!("eta at (1,4): step={}, complexity={}", eta.step, eta.complexity);

    let interval = DiscreteInterval::new(2, 7);
    let epsilon = SimpleAdjunction::counit_at(&interval);
    println!("epsilon at {interval}: {epsilon}");
    println!();
}

// ============================================================================
// Triangle Identity Verification
// ============================================================================

fn triangle_identities() {
    println!("=== Triangle Identities ===\n");

    // Build a computation sequence
    let states: Vec<ComputationState> = (0..6)
        .scan(ComputationState::initial(), |s, _| {
            *s = s.next();
            Some(s.clone())
        })
        .collect();

    println!("Verifying over {} states:", states.len());
    for s in &states {
        let t1 = SimpleAdjunction::verify_triangle_1(s);
        let t2_interval = SimpleAdjunction::zprime(s);
        let t2 = SimpleAdjunction::verify_triangle_2(&t2_interval);
        println!("  step={}: triangle_1={t1}, triangle_2={t2}", s.step);
    }
    println!();
}

// ============================================================================
// AdjunctionVerification
// ============================================================================

fn verification() {
    println!("=== AdjunctionVerification ===\n");

    let states: Vec<ComputationState> = (0..5)
        .scan(ComputationState::initial(), |s, _| {
            *s = s.next();
            Some(s.clone())
        })
        .collect();

    let result = AdjunctionVerification::verify_sequence::<SimpleAdjunction>(&states);
    println!("{result}");
    println!("Triangle 1 failures: {}", result.triangle_1_failures());
    println!("Triangle 2 failures: {}", result.triangle_2_failures());
    println!();
}

// ============================================================================
// AdjunctionIrreducibility
// ============================================================================

fn irreducibility() {
    println!("=== AdjunctionIrreducibility ===\n");

    let states: Vec<ComputationState> = (0..5)
        .scan(ComputationState::initial(), |s, _| {
            *s = s.next();
            Some(s.clone())
        })
        .collect();

    let indicator = SimpleAdjunction::adjunction_irreducibility_indicator(&states);
    println!("Irreducibility indicator = {indicator:.4}");

    for s in &states {
        let gap = SimpleAdjunction::adjunction_gap(s);
        println!("  step={}: adjunction_gap = {gap:.4}", s.step);
    }

    // Empty sequence
    let empty_indicator = SimpleAdjunction::adjunction_irreducibility_indicator(&[]);
    println!("\nEmpty sequence indicator = {empty_indicator:.4}");
    println!();
}

fn main() {
    functor_demo();
    unit_counit();
    triangle_identities();
    verification();
    irreducibility();
}
