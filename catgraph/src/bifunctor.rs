//! Bifunctor and tensor product operations for parallel intervals.
//!
//! This module provides the bifunctorial structure for [`ParallelIntervals`],
//! formalizing the tensor product ⊗ in the cobordism category ℬ.
//!
//! ## Mathematical Background
//!
//! In multiway systems, parallel computations form a tensor product structure:
//! - **Tensor product**: I₁ ⊗ I₂ represents two intervals executing in parallel
//! - **Bifunctor**: Maps (f, g) → (f(I₁), g(I₂)) independently on each component
//!
//! The symmetric monoidal structure satisfies:
//! - **Associativity**: (I₁ ⊗ I₂) ⊗ I₃ ≅ I₁ ⊗ (I₂ ⊗ I₃)
//! - **Unit**: I ⊗ ∅ ≅ I ≅ ∅ ⊗ I
//! - **Symmetry**: I₁ ⊗ I₂ ≅ I₂ ⊗ I₁

use crate::interval::{DiscreteInterval, ParallelIntervals};

// ============================================================================
// Tensor Product Operations
// ============================================================================

/// Trait for types that support tensor product (parallel composition).
///
/// This models the monoidal structure of parallel computations in the
/// cobordism category.
pub trait TensorProduct: Sized {
    /// The unit element for tensor product (empty/identity).
    fn unit() -> Self;

    /// Tensor product: combine two structures in parallel.
    #[must_use]
    fn tensor(self, other: Self) -> Self;

    /// Check if this is the unit element.
    fn is_unit(&self) -> bool;
}

impl TensorProduct for ParallelIntervals {
    /// The unit element is an empty parallel interval structure.
    fn unit() -> Self {
        ParallelIntervals::new()
    }

    /// Tensor product: merge all branches from both structures.
    ///
    /// The result contains all branches from `self` followed by all branches
    /// from `other`. This models parallel execution of multiple computation paths.
    fn tensor(mut self, other: Self) -> Self {
        self.branches.extend(other.branches);
        self
    }

    /// Check if this is the unit (no branches).
    fn is_unit(&self) -> bool {
        self.branches.is_empty()
    }
}

// ============================================================================
// Tensor Transformation
// ============================================================================

/// Transform both components of a tensor product independently.
///
/// This is the core bifunctor operation: given transformations for the "left"
/// and "right" components, apply them independently.
pub fn tensor_bimap<F, G>(
    left: ParallelIntervals,
    right: ParallelIntervals,
    f: F,
    g: G,
) -> (ParallelIntervals, ParallelIntervals)
where
    F: FnOnce(ParallelIntervals) -> ParallelIntervals,
    G: FnOnce(ParallelIntervals) -> ParallelIntervals,
{
    (f(left), g(right))
}

/// Transform only the left component, preserving the right.
pub fn tensor_first<F>(
    left: ParallelIntervals,
    right: ParallelIntervals,
    f: F,
) -> (ParallelIntervals, ParallelIntervals)
where
    F: FnOnce(ParallelIntervals) -> ParallelIntervals,
{
    (f(left), right)
}

/// Transform only the right component, preserving the left.
pub fn tensor_second<G>(
    left: ParallelIntervals,
    right: ParallelIntervals,
    g: G,
) -> (ParallelIntervals, ParallelIntervals)
where
    G: FnOnce(ParallelIntervals) -> ParallelIntervals,
{
    (left, g(right))
}

// ============================================================================
// Interval Transformations for ParallelIntervals
// ============================================================================

/// Extension trait for transforming intervals within a parallel structure.
///
/// These operations apply transformations to all branches in a
/// [`ParallelIntervals`], useful for bifunctor operations.
pub trait IntervalTransform {
    /// Shift all intervals by an offset.
    #[must_use]
    fn shift_all(self, offset: isize) -> Self;

    /// Scale all intervals by a factor.
    #[must_use]
    fn scale_all(self, factor: usize) -> Self;

    /// Map a function over all branches.
    #[must_use]
    fn map_branches<F>(self, f: F) -> Self
    where
        F: FnMut(DiscreteInterval) -> DiscreteInterval;
}

impl IntervalTransform for ParallelIntervals {
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    fn shift_all(mut self, offset: isize) -> Self {
        self.branches = self
            .branches
            .into_iter()
            .map(|interval| {
                let new_start = (interval.start as isize + offset).max(0) as usize;
                let new_end = (interval.end as isize + offset).max(0) as usize;
                DiscreteInterval::new(new_start, new_end)
            })
            .collect();
        self
    }

    fn scale_all(mut self, factor: usize) -> Self {
        self.branches = self
            .branches
            .into_iter()
            .map(|interval| {
                let length = interval.end - interval.start;
                DiscreteInterval::new(interval.start, interval.start + length * factor)
            })
            .collect();
        self
    }

    fn map_branches<F>(mut self, f: F) -> Self
    where
        F: FnMut(DiscreteInterval) -> DiscreteInterval,
    {
        self.branches = self.branches.into_iter().map(f).collect();
        self
    }
}

// ============================================================================
// Monoidal Structure Verification
// ============================================================================

/// Verify that the tensor product is associative for three parallel intervals.
///
/// Checks: (a ⊗ b) ⊗ c has the same branches as a ⊗ (b ⊗ c)
#[must_use]
pub fn verify_associativity(
    a: &ParallelIntervals,
    b: &ParallelIntervals,
    c: &ParallelIntervals,
) -> bool {
    let left_assoc = a.clone().tensor(b.clone()).tensor(c.clone());
    let right_assoc = a.clone().tensor(b.clone().tensor(c.clone()));

    left_assoc.branches == right_assoc.branches
}

/// Verify that the unit element is neutral for tensor product.
///
/// Checks: a ⊗ ∅ ≅ a ≅ ∅ ⊗ a
#[must_use]
pub fn verify_unit_laws(a: &ParallelIntervals) -> bool {
    let unit = ParallelIntervals::unit();

    let left_unit = unit.clone().tensor(a.clone());
    let right_unit = a.clone().tensor(unit);

    left_unit.branches == a.branches && right_unit.branches == a.branches
}

/// Verify that the tensor product is symmetric (up to reordering).
///
/// Checks: a ⊗ b has the same branches as b ⊗ a (modulo order)
#[must_use]
pub fn verify_symmetry(a: &ParallelIntervals, b: &ParallelIntervals) -> bool {
    let ab = a.clone().tensor(b.clone());
    let ba = b.clone().tensor(a.clone());

    // Same total count
    if ab.branches.len() != ba.branches.len() {
        return false;
    }

    // All branches from `ab` are present in `ba` (order may differ)
    let mut ab_set: Vec<_> = ab.branches.iter().map(|i| (i.start, i.end)).collect();
    let mut ba_set: Vec<_> = ba.branches.iter().map(|i| (i.start, i.end)).collect();
    ab_set.sort_unstable();
    ba_set.sort_unstable();

    ab_set == ba_set
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_interval(start: usize, end: usize) -> DiscreteInterval {
        DiscreteInterval::new(start, end)
    }

    fn make_parallel(intervals: Vec<(usize, usize)>) -> ParallelIntervals {
        let mut p = ParallelIntervals::new();
        for (s, e) in intervals {
            p.add_branch(make_interval(s, e));
        }
        p
    }

    #[test]
    fn test_tensor_product_unit() {
        let unit = ParallelIntervals::unit();
        assert!(unit.is_unit());
        assert!(unit.branches.is_empty());
    }

    #[test]
    fn test_tensor_product_combines_branches() {
        let a = make_parallel(vec![(0, 5), (10, 15)]);
        let b = make_parallel(vec![(20, 25)]);

        let ab = a.tensor(b);
        assert_eq!(ab.branches.len(), 3);
    }

    #[test]
    fn test_tensor_bimap() {
        let left = make_parallel(vec![(0, 5)]);
        let right = make_parallel(vec![(10, 15)]);

        let (new_left, new_right) = tensor_bimap(
            left,
            right,
            |p| p.shift_all(10),
            |p| p.scale_all(2),
        );

        assert_eq!(new_left.branches[0].start, 10);
        assert_eq!(new_left.branches[0].end, 15);
        assert_eq!(new_right.branches[0].start, 10);
        assert_eq!(new_right.branches[0].end, 20); // 10 + (15-10)*2
    }

    #[test]
    fn test_tensor_first() {
        let left = make_parallel(vec![(0, 5)]);
        let right = make_parallel(vec![(10, 15)]);

        let (new_left, new_right) = tensor_first(left, right, |p| p.shift_all(100));

        assert_eq!(new_left.branches[0].start, 100);
        assert_eq!(new_right.branches[0].start, 10); // unchanged
    }

    #[test]
    fn test_tensor_second() {
        let left = make_parallel(vec![(0, 5)]);
        let right = make_parallel(vec![(10, 15)]);

        let (new_left, new_right) = tensor_second(left, right, |p| p.shift_all(100));

        assert_eq!(new_left.branches[0].start, 0); // unchanged
        assert_eq!(new_right.branches[0].start, 110);
    }

    #[test]
    fn test_shift_all() {
        let p = make_parallel(vec![(0, 5), (10, 15)]);
        let shifted = p.shift_all(5);

        assert_eq!(shifted.branches[0].start, 5);
        assert_eq!(shifted.branches[0].end, 10);
        assert_eq!(shifted.branches[1].start, 15);
        assert_eq!(shifted.branches[1].end, 20);
    }

    #[test]
    fn test_scale_all() {
        let p = make_parallel(vec![(0, 5), (10, 15)]);
        let scaled = p.scale_all(2);

        assert_eq!(scaled.branches[0].start, 0);
        assert_eq!(scaled.branches[0].end, 10); // 0 + 5*2
        assert_eq!(scaled.branches[1].start, 10);
        assert_eq!(scaled.branches[1].end, 20); // 10 + 5*2
    }

    #[test]
    fn test_map_branches() {
        let p = make_parallel(vec![(0, 5), (10, 15)]);
        let mapped = p.map_branches(|i| DiscreteInterval::new(i.start + 1, i.end + 1));

        assert_eq!(mapped.branches[0].start, 1);
        assert_eq!(mapped.branches[0].end, 6);
        assert_eq!(mapped.branches[1].start, 11);
        assert_eq!(mapped.branches[1].end, 16);
    }

    #[test]
    fn test_verify_associativity() {
        let a = make_parallel(vec![(0, 5)]);
        let b = make_parallel(vec![(10, 15)]);
        let c = make_parallel(vec![(20, 25)]);

        assert!(verify_associativity(&a, &b, &c));
    }

    #[test]
    fn test_verify_unit_laws() {
        let a = make_parallel(vec![(0, 5), (10, 15)]);
        assert!(verify_unit_laws(&a));
    }

    #[test]
    fn test_verify_symmetry() {
        let a = make_parallel(vec![(0, 5)]);
        let b = make_parallel(vec![(10, 15)]);

        assert!(verify_symmetry(&a, &b));
    }

    #[test]
    fn test_symmetry_with_same_intervals() {
        let a = make_parallel(vec![(0, 5), (0, 5)]);
        let b = make_parallel(vec![(10, 15)]);

        assert!(verify_symmetry(&a, &b));
    }
}
