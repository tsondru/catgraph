//! Integration tests for the bifunctor module.
//!
//! Verifies monoidal structure (associativity, unit, symmetry) and
//! bifunctor operations (`tensor_bimap`, `tensor_first`, `tensor_second`)
//! on `ParallelIntervals`.

use catgraph::bifunctor::{
    tensor_bimap, tensor_first, tensor_second, verify_associativity, verify_symmetry,
    verify_unit_laws, IntervalTransform, TensorProduct,
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
// Monoidal axioms
// ---------------------------------------------------------------------------

#[test]
fn tensor_associativity() {
    let a = make_parallel(&[(0, 5)]);
    let b = make_parallel(&[(10, 15), (20, 25)]);
    let c = make_parallel(&[(30, 35)]);

    assert!(verify_associativity(&a, &b, &c));
}

#[test]
fn tensor_unit_laws() {
    let a = make_parallel(&[(0, 5), (10, 15)]);
    assert!(verify_unit_laws(&a));

    // Also check that the empty structure is recognized as unit.
    assert!(ParallelIntervals::unit().is_unit());
}

#[test]
fn tensor_symmetry() {
    let a = make_parallel(&[(0, 5)]);
    let b = make_parallel(&[(10, 15)]);
    assert!(verify_symmetry(&a, &b));

    // Duplicate branches in one operand.
    let c = make_parallel(&[(0, 5), (0, 5)]);
    assert!(verify_symmetry(&a, &c));
}

// ---------------------------------------------------------------------------
// Bifunctor operations
// ---------------------------------------------------------------------------

#[test]
fn tensor_bimap_independence() {
    let left = make_parallel(&[(0, 5)]);
    let right = make_parallel(&[(10, 15)]);

    let (new_left, new_right) =
        tensor_bimap(left, right, |p| p.shift_all(100), |p| p.scale_all(3));

    // Left was shifted by 100.
    assert_eq!(new_left.branches[0].start, 100);
    assert_eq!(new_left.branches[0].end, 105);

    // Right was scaled by 3: length 5 -> 15, so [10, 10+15] = [10, 25].
    assert_eq!(new_right.branches[0].start, 10);
    assert_eq!(new_right.branches[0].end, 25);
}

#[test]
fn tensor_first_only_affects_first() {
    let left = make_parallel(&[(0, 5)]);
    let right = make_parallel(&[(10, 15)]);

    let right_before = right.clone();
    let (new_left, new_right) = tensor_first(left, right, |p| p.shift_all(50));

    assert_eq!(new_left.branches[0].start, 50);
    assert_eq!(new_right.branches, right_before.branches);
}

#[test]
fn tensor_second_only_affects_second() {
    let left = make_parallel(&[(0, 5)]);
    let right = make_parallel(&[(10, 15)]);

    let left_before = left.clone();
    let (new_left, new_right) = tensor_second(left, right, |p| p.shift_all(50));

    assert_eq!(new_left.branches, left_before.branches);
    assert_eq!(new_right.branches[0].start, 60);
}
