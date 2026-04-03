//! Property-based tests for discrete interval algebraic laws.
//!
//! Verifies associativity, identity, commutativity, and transitivity laws
//! for `DiscreteInterval`, plus monoidal (tensor) laws for `ParallelIntervals`.

use catgraph::interval::{DiscreteInterval, ParallelIntervals};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate a valid `DiscreteInterval` with small bounds.
///
/// start in 0..=50, length in 0..=20 (so end = start + length).
fn arb_interval() -> impl Strategy<Value = DiscreteInterval> {
    (0_usize..=50, 0_usize..=20).prop_map(|(start, len)| DiscreteInterval::new(start, start + len))
}

/// Generate three composable intervals: [a,b], [b,c], [c,d].
///
/// Each gap (b-a), (c-b), (d-c) is independently chosen from 0..=10.
fn arb_three_composable() -> impl Strategy<Value = (DiscreteInterval, DiscreteInterval, DiscreteInterval)>
{
    (0_usize..=30, 0_usize..=10, 0_usize..=10, 0_usize..=10).prop_map(
        |(a, gap1, gap2, gap3)| {
            let b = a + gap1;
            let c = b + gap2;
            let d = c + gap3;
            (
                DiscreteInterval::new(a, b),
                DiscreteInterval::new(b, c),
                DiscreteInterval::new(c, d),
            )
        },
    )
}

/// Generate a `ParallelIntervals` with 0..=5 branches.
fn arb_parallel() -> impl Strategy<Value = ParallelIntervals> {
    prop::collection::vec(arb_interval(), 0..=5).prop_map(|branches| {
        let mut pi = ParallelIntervals::new();
        for b in branches {
            pi.add_branch(b);
        }
        pi
    })
}

// ---------------------------------------------------------------------------
// DiscreteInterval property tests
// ---------------------------------------------------------------------------

proptest! {
    /// Composition associativity: a.then(b).then(c) == a.then(b.then(c))
    ///
    /// For composable intervals [a,b], [b,c], [c,d]:
    ///   ([a,b].then([b,c])).then([c,d]) == [a,b].then([b,c].then([c,d]))
    /// Both sides should yield [a,d].
    #[test]
    fn composition_associativity((ab, bc, cd) in arb_three_composable()) {
        let left = ab.then(bc).unwrap().then(cd).unwrap();
        let right = ab.then(bc.then(cd).unwrap()).unwrap();
        prop_assert_eq!(left, right, "associativity: (ab;bc);cd != ab;(bc;cd)");
    }

    /// Left identity: identity(a).then([a,b]) == [a,b]
    #[test]
    fn left_identity_composition(interval in arb_interval()) {
        let id = DiscreteInterval::identity(interval.start);
        let result = id.then(interval).unwrap();
        prop_assert_eq!(result, interval, "id;f should equal f");
    }

    /// Right identity: [a,b].then(identity(b)) == [a,b]
    #[test]
    fn right_identity_composition(interval in arb_interval()) {
        let id = DiscreteInterval::identity(interval.end);
        let result = interval.then(id).unwrap();
        prop_assert_eq!(result, interval, "f;id should equal f");
    }

    /// Intersection commutativity: a.intersect(b) == b.intersect(a)
    #[test]
    fn intersection_commutativity(a in arb_interval(), b in arb_interval()) {
        let ab = a.intersect(&b);
        let ba = b.intersect(&a);
        prop_assert_eq!(ab, ba, "intersect should be commutative");
    }

    /// contains_interval transitivity: if a contains b and b contains c, then a contains c.
    #[test]
    fn contains_interval_transitivity(
        base in 0_usize..=20,
        outer_pad_left in 0_usize..=5,
        outer_pad_right in 0_usize..=5,
        mid_pad_left in 0_usize..=3,
        mid_pad_right in 0_usize..=3,
        inner_len in 0_usize..=4,
    ) {
        // Build nested intervals: outer >= mid >= inner
        let inner_start = base + outer_pad_left + mid_pad_left;
        let inner_end = inner_start + inner_len;
        let mid_start = inner_start - mid_pad_left;
        let mid_end = inner_end + mid_pad_right;
        let outer_start = mid_start - outer_pad_left;
        let outer_end = mid_end + outer_pad_right;

        let outer = DiscreteInterval::new(outer_start, outer_end);
        let mid = DiscreteInterval::new(mid_start, mid_end);
        let inner = DiscreteInterval::new(inner_start, inner_end);

        // Verify containment chain
        prop_assert!(outer.contains_interval(&mid), "outer should contain mid");
        prop_assert!(mid.contains_interval(&inner), "mid should contain inner");
        prop_assert!(
            outer.contains_interval(&inner),
            "transitivity: outer should contain inner"
        );
    }
}

// ---------------------------------------------------------------------------
// ParallelIntervals property tests
// ---------------------------------------------------------------------------

proptest! {
    /// Tensor right unit: x.tensor(empty) preserves x's branches exactly.
    #[test]
    fn tensor_right_unit(x in arb_parallel()) {
        let original_branches = x.branches.clone();
        let result = x.tensor(ParallelIntervals::new());
        prop_assert_eq!(
            result.branches, original_branches,
            "x tensor empty should equal x"
        );
    }

    /// Tensor associativity: (a tensor b) tensor c == a tensor (b tensor c)
    ///
    /// Both sides concatenate the same branches in the same order.
    #[test]
    fn tensor_associativity(
        a in arb_parallel(),
        b in arb_parallel(),
        c in arb_parallel(),
    ) {
        let left = a.clone().tensor(b.clone()).tensor(c.clone());
        let right = a.tensor(b.tensor(c));
        prop_assert_eq!(
            left.branches, right.branches,
            "tensor associativity: (a⊗b)⊗c != a⊗(b⊗c)"
        );
    }

    /// Branch count after tensor is the sum of the operand branch counts.
    #[test]
    fn tensor_branch_count_additive(a in arb_parallel(), b in arb_parallel()) {
        let expected = a.branch_count() + b.branch_count();
        let result = a.tensor(b);
        prop_assert_eq!(
            result.branch_count(),
            expected,
            "branch_count(a⊗b) should equal branch_count(a) + branch_count(b)"
        );
    }
}
