//! Symmetric monoidal coherence verification.
//!
//! Verifies that a collection of [`ParallelIntervals`] satisfies the coherence
//! conditions of a symmetric monoidal category:
//! - **Associator** α: (X ⊗ Y) ⊗ Z ≅ X ⊗ (Y ⊗ Z)
//! - **Left unitor** λ: I ⊗ X ≅ X
//! - **Right unitor** ρ: X ⊗ I ≅ X
//! - **Braiding** σ: X ⊗ Y ≅ Y ⊗ X
//!
//! Also provides [`DifferentialCoherence`] which interprets algebraic coherence
//! through a differential geometric lens (Stokes-like closure conditions).

use crate::interval::ParallelIntervals;

// ============================================================================
// Individual Coherence Checks
// ============================================================================

/// Verify associator coherence: α_{X,Y,Z}: (X ⊗ Y) ⊗ Z ≅ X ⊗ (Y ⊗ Z).
///
/// The associator proves that grouping in tensor products doesn't matter.
/// For parallel intervals, this checks structural equivalence of:
/// `(a ⊗ b) ⊗ c` vs `a ⊗ (b ⊗ c)`
#[must_use]
pub fn verify_associator_coherence(
    a: &ParallelIntervals,
    b: &ParallelIntervals,
    c: &ParallelIntervals,
) -> bool {
    // (a ⊗ b) ⊗ c
    let left_grouped = a.clone().tensor(b.clone()).tensor(c.clone());

    // a ⊗ (b ⊗ c)
    let right_grouped = a.clone().tensor(b.clone().tensor(c.clone()));

    // They should be structurally equivalent
    left_grouped.structurally_equivalent(&right_grouped)
}

/// Verify left unitor coherence: λ\_X: I ⊗ X ≅ X.
///
/// The left unitor proves that tensoring with the unit (empty) on the left
/// gives back the original structure.
#[must_use]
pub fn verify_left_unitor_coherence(x: &ParallelIntervals) -> bool {
    // I = empty ParallelIntervals (unit object)
    let unit = ParallelIntervals::new();

    // I ⊗ X should be equivalent to X
    let tensored = unit.tensor(x.clone());

    tensored.structurally_equivalent(x)
}

/// Verify right unitor coherence: ρ\_X: X ⊗ I ≅ X.
///
/// The right unitor proves that tensoring with the unit (empty) on the right
/// gives back the original structure.
#[must_use]
pub fn verify_right_unitor_coherence(x: &ParallelIntervals) -> bool {
    // I = empty ParallelIntervals (unit object)
    let unit = ParallelIntervals::new();

    // X ⊗ I should be equivalent to X
    let tensored = x.clone().tensor(unit);

    tensored.structurally_equivalent(x)
}

/// Verify braiding coherence: σ_{X,Y}: X ⊗ Y ≅ Y ⊗ X.
///
/// The braiding proves that the tensor product is symmetric - order doesn't matter.
#[must_use]
pub fn verify_braiding_coherence(x: &ParallelIntervals, y: &ParallelIntervals) -> bool {
    // X ⊗ Y
    let xy = x.clone().tensor(y.clone());

    // Y ⊗ X
    let yx = y.clone().tensor(x.clone());

    // They should be structurally equivalent (symmetric monoidal)
    xy.structurally_equivalent(&yx)
}

// ============================================================================
// Comprehensive Coherence Verification
// ============================================================================

/// Comprehensive coherence verification result.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug)]
pub struct CoherenceVerification {
    /// Associator coherence holds for all tested triples.
    pub associator_coherent: bool,
    /// Left unitor coherence holds.
    pub left_unitor_coherent: bool,
    /// Right unitor coherence holds.
    pub right_unitor_coherent: bool,
    /// Braiding coherence holds for all tested pairs.
    pub braiding_coherent: bool,
    /// Number of associator tests performed.
    pub associator_tests: usize,
    /// Number of braiding tests performed.
    pub braiding_tests: usize,
    /// Overall coherence (all conditions satisfied).
    pub fully_coherent: bool,
}

impl CoherenceVerification {
    /// Verify all coherence conditions for a collection of parallel intervals.
    ///
    /// Tests associator with all triples and braiding with all pairs.
    pub fn verify_all(intervals: &[ParallelIntervals]) -> Self {
        if intervals.is_empty() {
            return Self {
                associator_coherent: true,
                left_unitor_coherent: true,
                right_unitor_coherent: true,
                braiding_coherent: true,
                associator_tests: 0,
                braiding_tests: 0,
                fully_coherent: true,
            };
        }

        // Test all elements for unitors
        let all_left_unitor = intervals.iter().all(verify_left_unitor_coherence);
        let all_right_unitor = intervals.iter().all(verify_right_unitor_coherence);

        // Test associator on all triples
        let mut associator_tests = 0;
        let mut associator_coherent = true;
        for i in 0..intervals.len() {
            for j in 0..intervals.len() {
                for k in 0..intervals.len() {
                    associator_tests += 1;
                    if !verify_associator_coherence(&intervals[i], &intervals[j], &intervals[k]) {
                        associator_coherent = false;
                    }
                }
            }
        }

        // Test braiding on all pairs
        let mut braiding_tests = 0;
        let mut braiding_coherent = true;
        for i in 0..intervals.len() {
            for j in 0..intervals.len() {
                braiding_tests += 1;
                if !verify_braiding_coherence(&intervals[i], &intervals[j]) {
                    braiding_coherent = false;
                }
            }
        }

        let fully_coherent = associator_coherent
            && all_left_unitor
            && all_right_unitor
            && braiding_coherent;

        Self {
            associator_coherent,
            left_unitor_coherent: all_left_unitor,
            right_unitor_coherent: all_right_unitor,
            braiding_coherent,
            associator_tests,
            braiding_tests,
            fully_coherent,
        }
    }
}

impl std::fmt::Display for CoherenceVerification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Coherence Verification:")?;
        writeln!(f, "  Fully coherent: {}", self.fully_coherent)?;
        writeln!(
            f,
            "  Associator α: {} ({} tests)",
            self.associator_coherent, self.associator_tests
        )?;
        writeln!(f, "  Left unitor λ: {}", self.left_unitor_coherent)?;
        writeln!(f, "  Right unitor ρ: {}", self.right_unitor_coherent)?;
        writeln!(
            f,
            "  Braiding σ: {} ({} tests)",
            self.braiding_coherent, self.braiding_tests
        )
    }
}

// ============================================================================
// Differential Coherence
// ============================================================================

/// Differential geometric interpretation of coherence conditions.
///
/// Connects algebraic coherence (pentagon identity, triangle identities)
/// with differential geometry via the Stokes perspective.
///
/// - **Pentagon identity**: Viewed as closure of a 2-form (d²ω = 0)
/// - **Triangle identities**: Viewed as conservation on 1-chains
/// - **Coherence form closure**: If dω = 0, coherence is "conserved"
///
/// When algebraic coherence holds AND the differential form is closed,
/// we have *differential coherence* — a geometric confirmation of the
/// categorical structure.
#[derive(Clone, Debug)]
pub struct DifferentialCoherence {
    /// Standard algebraic coherence verification.
    pub algebraic: CoherenceVerification,

    /// Whether the differential coherence criterion is satisfied.
    pub differential_coherent: bool,

    /// Whether the "coherence form" (encoding coherence data) is closed.
    pub coherence_form_closed: bool,

    /// Conservation ratio from Stokes analysis.
    ///
    /// Ratio = 1.0 means perfect conservation (fully coherent).
    /// Deviations indicate coherence "leakage".
    pub conservation_ratio: f64,

    /// Number of coherence violations detected as non-closures.
    pub non_closure_count: usize,
}

impl DifferentialCoherence {
    /// Verify differential coherence for a collection of parallel intervals.
    ///
    /// Performs both algebraic coherence verification and interprets
    /// the results through the lens of differential forms and Stokes theorem.
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    #[must_use]
    pub fn verify(intervals: &[ParallelIntervals]) -> Self {
        // Step 1: Standard algebraic coherence
        let algebraic = CoherenceVerification::verify_all(intervals);

        // Step 2: Interpret coherence through differential lens
        let coherence_form_closed = algebraic.associator_coherent;

        // Step 3: Compute conservation ratio
        let total_tests = algebraic.associator_tests + algebraic.braiding_tests;
        let passed_tests = if algebraic.associator_coherent {
            algebraic.associator_tests
        } else {
            0
        } + if algebraic.braiding_coherent {
            algebraic.braiding_tests
        } else {
            0
        };

        let conservation_ratio = if total_tests == 0 {
            1.0
        } else {
            passed_tests as f64 / total_tests as f64
        };

        // Step 4: Count non-closures
        let mut non_closure_count = 0;
        if !algebraic.associator_coherent {
            non_closure_count += 1; // Pentagon identity violation
        }
        if !algebraic.left_unitor_coherent || !algebraic.right_unitor_coherent {
            non_closure_count += 1; // Triangle identity violation
        }
        if !algebraic.braiding_coherent {
            non_closure_count += 1; // Hexagon identity violation
        }

        // Step 5: Differential coherence requires both algebraic and form closure
        let differential_coherent =
            algebraic.fully_coherent && coherence_form_closed && conservation_ratio > 0.999;

        Self {
            algebraic,
            differential_coherent,
            coherence_form_closed,
            conservation_ratio,
            non_closure_count,
        }
    }

    /// Check if the structure exhibits "curvature" in category space.
    ///
    /// Non-zero curvature indicates coherence failures — the category
    /// is not "flat" in the sense that parallel transport (composition)
    /// is path-dependent.
    #[inline]
    #[must_use]
    pub fn has_categorical_curvature(&self) -> bool {
        self.non_closure_count > 0 || !self.coherence_form_closed
    }

    /// Compute the "coherence defect" as a single scalar.
    ///
    /// Analogous to scalar curvature in Riemannian geometry.
    /// A value of 0.0 indicates perfect coherence (flat category).
    #[inline]
    #[must_use]
    pub fn coherence_defect(&self) -> f64 {
        1.0 - self.conservation_ratio
    }
}

impl std::fmt::Display for DifferentialCoherence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Differential Coherence:")?;
        writeln!(
            f,
            "  Differentially coherent: {}",
            self.differential_coherent
        )?;
        writeln!(f, "  Coherence form closed: {}", self.coherence_form_closed)?;
        writeln!(f, "  Conservation ratio: {:.4}", self.conservation_ratio)?;
        writeln!(f, "  Non-closure count: {}", self.non_closure_count)?;
        writeln!(
            f,
            "  Categorical curvature: {}",
            if self.has_categorical_curvature() {
                "present"
            } else {
                "flat"
            }
        )?;
        writeln!(f)?;
        write!(f, "{}", self.algebraic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interval::DiscreteInterval;

    fn make_parallel(intervals: Vec<(usize, usize)>) -> ParallelIntervals {
        let mut p = ParallelIntervals::new();
        for (s, e) in intervals {
            p.add_branch(DiscreteInterval::new(s, e));
        }
        p
    }

    #[test]
    fn test_associator_coherence() {
        let a = make_parallel(vec![(0, 2)]);
        let b = make_parallel(vec![(2, 5)]);
        let c = make_parallel(vec![(5, 8)]);
        assert!(verify_associator_coherence(&a, &b, &c));
    }

    #[test]
    fn test_left_unitor_coherence() {
        let x = make_parallel(vec![(0, 5)]);
        assert!(verify_left_unitor_coherence(&x));
    }

    #[test]
    fn test_right_unitor_coherence() {
        let x = make_parallel(vec![(0, 5)]);
        assert!(verify_right_unitor_coherence(&x));
    }

    #[test]
    fn test_braiding_coherence() {
        let x = make_parallel(vec![(0, 3)]);
        let y = make_parallel(vec![(3, 7)]);
        assert!(verify_braiding_coherence(&x, &y));
    }

    #[test]
    fn test_coherence_verification_all() {
        let intervals = vec![
            make_parallel(vec![(0, 2)]),
            make_parallel(vec![(2, 5)]),
            make_parallel(vec![(5, 10)]),
        ];
        let result = CoherenceVerification::verify_all(&intervals);
        assert!(result.fully_coherent);
        assert_eq!(result.associator_tests, 27); // 3^3
        assert_eq!(result.braiding_tests, 9);     // 3^2
    }

    #[test]
    fn test_coherence_verification_empty() {
        let result = CoherenceVerification::verify_all(&[]);
        assert!(result.fully_coherent);
        assert_eq!(result.associator_tests, 0);
        assert_eq!(result.braiding_tests, 0);
    }

    #[test]
    fn test_differential_coherence() {
        let intervals = vec![
            make_parallel(vec![(0, 2)]),
            make_parallel(vec![(2, 5)]),
        ];
        let result = DifferentialCoherence::verify(&intervals);
        assert!(result.differential_coherent);
        assert!(result.coherence_form_closed);
        assert!(!result.has_categorical_curvature());
        assert!(result.coherence_defect() < 0.001);
    }

    #[test]
    fn test_differential_coherence_empty() {
        let result = DifferentialCoherence::verify(&[]);
        assert!(result.differential_coherent);
    }

    #[test]
    fn test_coherence_display() {
        let intervals = vec![make_parallel(vec![(0, 2)]), make_parallel(vec![(2, 4)])];
        let result = CoherenceVerification::verify_all(&intervals);
        let display = format!("{result}");
        assert!(display.contains("Fully coherent: true"));
        assert!(display.contains("Associator α: true"));
    }

    #[test]
    fn test_differential_coherence_display() {
        let intervals = vec![make_parallel(vec![(0, 2)]), make_parallel(vec![(2, 4)])];
        let result = DifferentialCoherence::verify(&intervals);
        let display = format!("{result}");
        assert!(display.contains("Differentially coherent: true"));
        assert!(display.contains("Categorical curvature: flat"));
    }
}
