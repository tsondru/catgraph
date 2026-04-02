//! Computation state representation for category 𝒯.
//!
//! A computation state is an object in the computation category,
//! representing a snapshot at a particular step with associated complexity.

use crate::interval::DiscreteInterval;

/// A computation state in category 𝒯.
///
/// Represents an object in the computation category - a snapshot of a
/// computation at a particular step with associated complexity.
///
/// # Mathematical Interpretation
///
/// In the functorial irreducibility framework:
/// - Objects in 𝒯 are computation states (TM configurations, CA generations, etc.)
/// - Morphisms in 𝒯 are transitions between states
/// - The functor Z': 𝒯 → ℬ maps states to time steps and transitions to intervals
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComputationState {
    /// The step number in the computation
    pub step: usize,
    /// The complexity (number of steps taken to reach this state)
    pub complexity: usize,
    /// Optional fingerprint for cycle detection
    pub fingerprint: Option<u64>,
}

impl ComputationState {
    /// Create a new computation state.
    #[must_use]
    pub fn new(step: usize, complexity: usize) -> Self {
        Self {
            step,
            complexity,
            fingerprint: None,
        }
    }

    /// Create a computation state with a fingerprint.
    #[must_use]
    pub fn with_fingerprint(step: usize, complexity: usize, fingerprint: u64) -> Self {
        Self {
            step,
            complexity,
            fingerprint: Some(fingerprint),
        }
    }

    /// Create the initial computation state.
    #[must_use]
    pub fn initial() -> Self {
        Self::new(0, 0)
    }

    /// Advance to the next state with one step of complexity.
    #[must_use]
    pub fn next(&self) -> Self {
        Self::new(self.step + 1, self.complexity + 1)
    }

    /// Convert to the corresponding interval in ℬ.
    ///
    /// Maps this computation state to the interval [step, step + complexity].
    /// When complexity is zero, produces a 1-step interval [step, step + 1]
    /// because every computation state occupies at least one time step in ℬ.
    #[must_use]
    pub fn to_interval(&self) -> DiscreteInterval {
        DiscreteInterval::new(self.step, self.step + self.complexity.max(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let state = ComputationState::new(3, 7);
        assert_eq!(state.step, 3);
        assert_eq!(state.complexity, 7);
        assert_eq!(state.fingerprint, None);
    }

    #[test]
    fn test_with_fingerprint() {
        let state = ComputationState::with_fingerprint(1, 2, 0xDEAD);
        assert_eq!(state.step, 1);
        assert_eq!(state.complexity, 2);
        assert_eq!(state.fingerprint, Some(0xDEAD));
    }

    #[test]
    fn test_initial() {
        let state = ComputationState::initial();
        assert_eq!(state.step, 0);
        assert_eq!(state.complexity, 0);
        assert_eq!(state.fingerprint, None);
    }

    #[test]
    fn test_next() {
        let s0 = ComputationState::new(0, 0);
        let s1 = s0.next();
        assert_eq!(s1.step, 1);
        assert_eq!(s1.complexity, 1);
        let s2 = s1.next();
        assert_eq!(s2.step, 2);
        assert_eq!(s2.complexity, 2);
    }

    #[test]
    fn test_to_interval() {
        let state = ComputationState::new(2, 5);
        let interval = state.to_interval();
        assert_eq!(interval.start, 2);
        assert_eq!(interval.end, 7);
    }

    #[test]
    fn test_to_interval_zero_complexity_uses_min_one_step() {
        // Zero complexity still produces a 1-step interval
        let state = ComputationState::new(3, 0);
        let interval = state.to_interval();
        assert_eq!(interval.start, 3);
        assert_eq!(interval.end, 4);
        assert_eq!(interval.steps(), 1);
    }

    #[test]
    fn test_equality() {
        let a = ComputationState::new(1, 2);
        let b = ComputationState::new(1, 2);
        let c = ComputationState::new(1, 3);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_clone() {
        let state = ComputationState::with_fingerprint(5, 10, 42);
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn test_hash_consistent_with_equality() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ComputationState::new(1, 2));
        set.insert(ComputationState::new(1, 2));
        assert_eq!(set.len(), 1);
        set.insert(ComputationState::new(2, 2));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_next_preserves_no_fingerprint() {
        let state = ComputationState::with_fingerprint(0, 0, 99);
        let next = state.next();
        // next() creates via new(), so no fingerprint
        assert_eq!(next.fingerprint, None);
    }
}
