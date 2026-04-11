//! Complexity algebra for computational processes.
//!
//! A complexity measure captures the "cost" of computation in terms of steps,
//! resources, or other metrics. The [`Complexity`] trait abstracts over different
//! cost models (time, space, energy) by requiring sequential composition (g∘f)
//! and parallel composition (f⊗g) operations.
//!
//! [`StepCount`] is the canonical implementation: sequential composition adds
//! steps, parallel composition takes the maximum (wall-clock time model).
//!
//! See also `examples/complexity.rs`.

use std::fmt;
use std::ops::{Add, AddAssign};

/// Trait for complexity measures on morphisms.
///
/// A complexity algebra must support:
/// - Sequential composition: complexity of g∘f
/// - Parallel composition: complexity of f⊗g
/// - Conversion to step count
pub trait Complexity: Clone + Default + fmt::Debug {
    /// Sequential composition: complexity of performing f then g.
    ///
    /// For most models, this is addition: C(g∘f) = C(f) + C(g)
    #[must_use]
    fn sequential(&self, other: &Self) -> Self;

    /// Parallel composition: complexity of performing f and g simultaneously.
    ///
    /// For time complexity, this is max: C(f⊗g) = max(C(f), C(g))
    /// For space complexity, this might be sum: C(f⊗g) = C(f) + C(g)
    #[must_use]
    fn parallel(&self, other: &Self) -> Self;

    /// Convert to a numeric step count.
    fn as_steps(&self) -> usize;

    /// Check if this is zero/identity complexity.
    fn is_zero(&self) -> bool {
        self.as_steps() == 0
    }
}

/// Simple step-counting complexity measure (wall-clock time model).
///
/// Represents the number of elementary computational steps. Sequential
/// composition adds steps (`C(g∘f) = C(f) + C(g)`), parallel composition
/// takes the maximum (`C(f⊗g) = max(C(f), C(g))`). Implements `Add` and
/// `AddAssign` for convenient arithmetic.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StepCount(pub usize);

impl StepCount {
    /// Create a new step count.
    #[must_use]
    pub fn new(steps: usize) -> Self {
        Self(steps)
    }

    /// Zero steps (identity complexity).
    #[must_use]
    pub fn zero() -> Self {
        Self(0)
    }

    /// Single step.
    #[must_use]
    pub fn one() -> Self {
        Self(1)
    }

    /// Get the raw step count.
    #[must_use]
    pub fn get(&self) -> usize {
        self.0
    }
}

impl Complexity for StepCount {
    fn sequential(&self, other: &Self) -> Self {
        StepCount(self.0 + other.0)
    }

    fn parallel(&self, other: &Self) -> Self {
        StepCount(self.0.max(other.0))
    }

    fn as_steps(&self) -> usize {
        self.0
    }
}

impl Add for StepCount {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        StepCount(self.0 + other.0)
    }
}

impl AddAssign for StepCount {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl From<usize> for StepCount {
    fn from(n: usize) -> Self {
        StepCount(n)
    }
}

impl From<StepCount> for usize {
    fn from(sc: StepCount) -> Self {
        sc.0
    }
}

impl fmt::Display for StepCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} step{}", self.0, if self.0 == 1 { "" } else { "s" })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_count_sequential() {
        let a = StepCount(3);
        let b = StepCount(5);
        assert_eq!(a.sequential(&b), StepCount(8));
    }

    #[test]
    fn test_step_count_parallel() {
        let a = StepCount(3);
        let b = StepCount(5);
        assert_eq!(a.parallel(&b), StepCount(5));
    }

    #[test]
    fn test_step_count_add() {
        let a = StepCount(3);
        let b = StepCount(5);
        assert_eq!(a + b, StepCount(8));
    }

    #[test]
    fn test_step_count_display() {
        assert_eq!(StepCount(1).to_string(), "1 step");
        assert_eq!(StepCount(5).to_string(), "5 steps");
    }

    #[test]
    fn test_step_count_zero() {
        let z = StepCount::zero();
        assert!(z.is_zero());
        assert_eq!(z.get(), 0);
    }
}
