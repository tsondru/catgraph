//! TensorProduct, IntervalTransform, and monoidal verification API demonstration.
//!
//! Shows the TensorProduct trait, bifunctorial mapping (tensor_bimap/first/second),
//! IntervalTransform operations (shift, scale, map_branches), and associativity/
//! unit/symmetry verification.

use catgraph::bifunctor::{
    tensor_bimap, tensor_first, tensor_second, verify_associativity, verify_symmetry,
    verify_unit_laws, IntervalTransform, TensorProduct,
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
// TensorProduct Trait
// ============================================================================

fn tensor_product_trait() {
    println!("=== TensorProduct Trait ===\n");

    let unit = ParallelIntervals::unit();
    println!("unit().is_unit() = {}", unit.is_unit());

    let a = make_parallel(vec![(0, 5), (10, 15)]);
    let b = make_parallel(vec![(20, 25)]);
    println!("a: {} branches, b: {} branches", a.branch_count(), b.branch_count());

    let ab = a.clone().tensor(b.clone());
    println!("a.tensor(b): {} branches", ab.branch_count());

    // Unit laws via trait
    let left_unit = ParallelIntervals::unit().tensor(a.clone());
    let right_unit = a.clone().tensor(ParallelIntervals::unit());
    println!(
        "unit ⊗ a == a: {}, a ⊗ unit == a: {}",
        left_unit.exactly_equal(&a),
        right_unit.exactly_equal(&a),
    );
    println!();
}

// ============================================================================
// IntervalTransform
// ============================================================================

fn interval_transform() {
    println!("=== IntervalTransform ===\n");

    let p = make_parallel(vec![(0, 5), (10, 15)]);
    println!("original: {:?}", p.branches);

    // shift_all
    let shifted = p.clone().shift_all(3);
    println!("shift_all(3): {:?}", shifted.branches);

    // Negative shift (clamped to 0)
    let neg_shifted = p.clone().shift_all(-20);
    println!("shift_all(-20): {:?}  (clamped to 0)", neg_shifted.branches);

    // scale_all
    let scaled = p.clone().scale_all(2);
    println!("scale_all(2): {:?}", scaled.branches);

    // map_branches: add 1 to each endpoint
    let mapped = p.map_branches(|i| DiscreteInterval::new(i.start + 1, i.end + 1));
    println!("map_branches(+1): {:?}", mapped.branches);
    println!();
}

// ============================================================================
// Bifunctor Mapping
// ============================================================================

fn bifunctor_mapping() {
    println!("=== Bifunctor Mapping ===\n");

    let left = make_parallel(vec![(0, 5)]);
    let right = make_parallel(vec![(10, 15)]);

    // tensor_bimap: transform both independently
    let (new_left, new_right) = tensor_bimap(
        left.clone(),
        right.clone(),
        |p| p.shift_all(10),
        |p| p.scale_all(2),
    );
    println!("tensor_bimap(shift_all(10), scale_all(2)):");
    println!("  left:  {:?}", new_left.branches);
    println!("  right: {:?}", new_right.branches);

    // tensor_first: transform only left
    let (new_left, new_right) = tensor_first(
        left.clone(),
        right.clone(),
        |p| p.shift_all(100),
    );
    println!("tensor_first(shift_all(100)):");
    println!("  left:  {:?}", new_left.branches);
    println!("  right: {:?}  (unchanged)", new_right.branches);

    // tensor_second: transform only right
    let (new_left, new_right) = tensor_second(left, right, |p| p.shift_all(100));
    println!("tensor_second(shift_all(100)):");
    println!("  left:  {:?}  (unchanged)", new_left.branches);
    println!("  right: {:?}", new_right.branches);
    println!();
}

// ============================================================================
// Monoidal Structure Verification
// ============================================================================

fn monoidal_verification() {
    println!("=== Monoidal Structure Verification ===\n");

    let a = make_parallel(vec![(0, 5)]);
    let b = make_parallel(vec![(10, 15)]);
    let c = make_parallel(vec![(20, 25)]);

    println!("verify_associativity(a, b, c) = {}", verify_associativity(&a, &b, &c));
    println!("verify_unit_laws(a)           = {}", verify_unit_laws(&a));
    println!("verify_symmetry(a, b)         = {}", verify_symmetry(&a, &b));

    // Multi-branch
    let multi = make_parallel(vec![(0, 2), (3, 5), (6, 8)]);
    println!("\nMulti-branch (3 branches):");
    println!("verify_unit_laws(multi) = {}", verify_unit_laws(&multi));
    println!("verify_symmetry(a, multi) = {}", verify_symmetry(&a, &multi));
    println!();
}

fn main() {
    tensor_product_trait();
    interval_transform();
    bifunctor_mapping();
    monoidal_verification();
}
