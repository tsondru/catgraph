//! Abstract adjunction framework for computation ⊣ cobordism categories.
//!
//! Provides the generic traits and verification types for adjunctions
//! between a computation category 𝒯 and a cobordism category ℬ.
//!
//! ## Mathematical Foundation
//!
//! An adjunction Z' ⊣ Z provides a formal relationship:
//!
//! ```text
//! Hom_ℬ(Z'(c), i) ≅ Hom_𝒯(c, Z(i))
//! ```
//!
//! where:
//! - Z': 𝒯 → ℬ maps computation states to intervals
//! - Z: ℬ → 𝒯 maps intervals back to computation states
//!
//! The **unit** η: `Id_𝒯` → Z∘Z' embeds computations into the adjunction.
//! The **counit** ε: Z'∘Z → `Id_ℬ` extracts intervals from the adjunction.
//!
//! ## Triangle Identities
//!
//! The adjunction satisfies two triangle identities:
//! 1. `ε_{Z'(c)}` ∘ Z'(`η_c`) = id_{Z'(c)}
//! 2. Z(`ε_i`) ∘ `η_{Z(i)}` = id_{Z(i)}

use crate::computation_state::ComputationState;
use crate::interval::DiscreteInterval;

/// Operations for a Z' ⊣ Z adjunction between computation and cobordism categories.
///
/// Implementors provide the concrete Z' and Z functors plus unit/counit
/// natural transformations and triangle identity verification.
pub trait ZPrimeOps {
    /// Apply Z': 𝒯 → ℬ (computation state to interval).
    fn zprime(state: &ComputationState) -> DiscreteInterval;

    /// Apply Z: ℬ → 𝒯 (interval to computation state).
    fn z(interval: &DiscreteInterval) -> ComputationState;

    /// Apply the unit at a specific state: `η_c`: c → Z(Z'(c)).
    fn unit_at(state: &ComputationState) -> ComputationState;

    /// Apply the counit at a specific interval: `ε_i`: Z'(Z(i)) → i.
    fn counit_at(interval: &DiscreteInterval) -> DiscreteInterval;

    /// Verify the first triangle identity: `ε_{Z'(c)}` ∘ Z'(`η_c`) = id_{Z'(c)}.
    fn verify_triangle_1(state: &ComputationState) -> bool;

    /// Verify the second triangle identity: Z(`ε_i`) ∘ `η_{Z(i)}` = id_{Z(i)}.
    fn verify_triangle_2(interval: &DiscreteInterval) -> bool;
}

/// Result of adjunction verification for a sequence of computations.
#[derive(Clone, Debug)]
pub struct AdjunctionVerification {
    /// Whether all triangle identities hold.
    pub triangle_identities_hold: bool,
    /// Results for triangle identity 1 at each state.
    pub triangle_1_results: Vec<bool>,
    /// Results for triangle identity 2 at each interval.
    pub triangle_2_results: Vec<bool>,
    /// Whether Z and Z' form an adjoint pair.
    pub is_adjoint_pair: bool,
}

impl AdjunctionVerification {
    /// Verify the adjunction for a sequence of computation states,
    /// using the given `ZPrimeOps` implementation.
    pub fn verify_sequence<T: ZPrimeOps>(states: &[ComputationState]) -> Self {
        let intervals: Vec<DiscreteInterval> = states.iter().map(T::zprime).collect();

        let triangle_1_results: Vec<bool> =
            states.iter().map(T::verify_triangle_1).collect();

        let triangle_2_results: Vec<bool> =
            intervals.iter().map(T::verify_triangle_2).collect();

        let all_triangle_1 = triangle_1_results.iter().all(|&b| b);
        let all_triangle_2 = triangle_2_results.iter().all(|&b| b);
        let triangle_identities_hold = all_triangle_1 && all_triangle_2;

        Self {
            triangle_identities_hold,
            triangle_1_results,
            triangle_2_results,
            is_adjoint_pair: triangle_identities_hold,
        }
    }

    /// Count failures in triangle identity 1.
    #[must_use]
    pub fn triangle_1_failures(&self) -> usize {
        self.triangle_1_results.iter().filter(|&&b| !b).count()
    }

    /// Count failures in triangle identity 2.
    #[must_use]
    pub fn triangle_2_failures(&self) -> usize {
        self.triangle_2_results.iter().filter(|&&b| !b).count()
    }
}

impl std::fmt::Display for AdjunctionVerification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Adjunction Verification:")?;
        writeln!(
            f,
            "  Triangle identities hold: {}",
            self.triangle_identities_hold
        )?;
        writeln!(
            f,
            "  Triangle 1 (ε ∘ Z'η = id): {}/{} passed",
            self.triangle_1_results.iter().filter(|&&b| b).count(),
            self.triangle_1_results.len()
        )?;
        writeln!(
            f,
            "  Triangle 2 (Zε ∘ η = id): {}/{} passed",
            self.triangle_2_results.iter().filter(|&&b| b).count(),
            self.triangle_2_results.len()
        )?;
        writeln!(f, "  Is adjoint pair: {}", self.is_adjoint_pair)
    }
}

/// Extension trait connecting the adjunction to irreducibility analysis.
///
/// Requires `ZPrimeOps` so that default implementations can compute
/// the adjunction gap via `unit_at`.
pub trait AdjunctionIrreducibility: ZPrimeOps {
    /// Check if the adjunction structure reveals irreducibility.
    ///
    /// The adjunction Z' ⊣ Z is "non-trivial" when the unit and counit
    /// are not identity transformations, which corresponds to the
    /// computation being irreducible.
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn adjunction_irreducibility_indicator(states: &[ComputationState]) -> f64 {
        if states.is_empty() {
            return 0.0;
        }
        let total_gap: f64 = states.iter().map(Self::adjunction_gap).sum();
        total_gap / states.len() as f64
    }

    /// Compute the "adjunction gap" - deviation from perfect adjoint pair.
    ///
    /// A gap of 0 indicates perfect adjunction (possibly reducible).
    /// A non-zero gap indicates the adjunction structure has "resistance"
    /// to composition, correlating with irreducibility.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn adjunction_gap(state: &ComputationState) -> f64 {
        let unit_result = Self::unit_at(state);
        let step_diff = (unit_result.step as f64 - state.step as f64).abs();
        let complexity_diff = (unit_result.complexity as f64 - state.complexity as f64).abs();
        step_diff + complexity_diff
    }
}
