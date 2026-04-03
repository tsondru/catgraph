//! ComputationState API demonstration.
//!
//! Shows all constructors, multi-step walk via `next()`, `to_interval()` mapping,
//! and equality/hashing behavior.

use std::collections::HashSet;

use catgraph::computation_state::ComputationState;

// ============================================================================
// Constructors
// ============================================================================

fn constructors() {
    println!("=== Constructors ===\n");

    let s = ComputationState::new(3, 7);
    println!("new(3, 7)              step={}, complexity={}, fingerprint={:?}", s.step, s.complexity, s.fingerprint);

    let fp = ComputationState::with_fingerprint(1, 2, 0xCAFE);
    println!("with_fingerprint(1,2,0xCAFE) step={}, complexity={}, fingerprint={:?}", fp.step, fp.complexity, fp.fingerprint);

    let init = ComputationState::initial();
    println!("initial()              step={}, complexity={}", init.step, init.complexity);
    println!();
}

// ============================================================================
// Multi-Step Walk
// ============================================================================

fn multi_step_walk() {
    println!("=== Multi-Step Walk ===\n");

    let s0 = ComputationState::initial();
    let s1 = s0.next();
    let s2 = s1.next();
    let s3 = s2.next();

    for (label, state) in [("s0", &s0), ("s1", &s1), ("s2", &s2), ("s3", &s3)] {
        println!("{label}: step={}, complexity={}", state.step, state.complexity);
    }

    // next() from a fingerprinted state drops the fingerprint
    let fp_state = ComputationState::with_fingerprint(5, 10, 0xDEAD);
    let after = fp_state.next();
    println!(
        "\nwith_fingerprint.next() => step={}, fingerprint={:?}  (fingerprint dropped)",
        after.step, after.fingerprint,
    );
    println!();
}

// ============================================================================
// to_interval Mapping
// ============================================================================

fn to_interval_mapping() {
    println!("=== to_interval Mapping ===\n");

    // Normal case: [step, step + complexity]
    let s = ComputationState::new(2, 5);
    let i = s.to_interval();
    println!("state(2, 5)  => interval {i}  (cardinality {})", i.cardinality());

    // Zero complexity: uses min 1 step => [step, step + 1]
    let zero = ComputationState::new(3, 0);
    let iz = zero.to_interval();
    println!("state(3, 0)  => interval {iz}  (min 1-step, cardinality {})", iz.cardinality());

    // Walk and map each state
    println!("\nWalk trajectory as intervals:");
    let mut state = ComputationState::initial();
    for _ in 0..5 {
        state = state.next();
        let interval = state.to_interval();
        println!("  step={}, complexity={} => {interval}", state.step, state.complexity);
    }
    println!();
}

// ============================================================================
// Equality and Hashing
// ============================================================================

fn equality_and_hashing() {
    println!("=== Equality & Hashing ===\n");

    let a = ComputationState::new(1, 2);
    let b = ComputationState::new(1, 2);
    let c = ComputationState::new(1, 3);
    println!("a=new(1,2), b=new(1,2), c=new(1,3)");
    println!("a == b  = {}", a == b);
    println!("a == c  = {}", a == c);

    // Clone preserves equality
    let d = a.clone();
    println!("a == a.clone() = {}", a == d);

    // HashSet deduplication
    let mut set = HashSet::new();
    set.insert(ComputationState::new(1, 2));
    set.insert(ComputationState::new(1, 2)); // duplicate
    set.insert(ComputationState::new(2, 2));
    println!("\nHashSet with (1,2), (1,2), (2,2) => len={}", set.len());
    println!();
}

fn main() {
    constructors();
    multi_step_walk();
    to_interval_mapping();
    equality_and_hashing();
}
