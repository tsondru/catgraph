//! Magnitude functions — Phase 6A.2 (Tsallis entropy + Möbius inversion).
//!
//! Phase 6A.3 will add the magnitude function itself on top of these two
//! primitives.
//!
//! - [`tsallis_entropy`] — Tsallis q-entropy with Shannon-recovery special
//!   case at `t = 1` (BV 2025 §3 / Tsallis 1988).
//! - [`mobius_function`] — Möbius inversion `ζ · μ = I` over a ring (Leinster
//!   2013 / Leinster-Shulman §2). v0.1.0 implements the matrix-inverse path
//!   via Gaussian elimination, requiring `Q: Ring + Div + From<f64>`.

use std::ops::Div;

use catgraph::errors::CatgraphError;

use crate::weighted_cospan::NodeId;
use crate::{LawvereMetricSpace, Ring, TSALLIS_SHANNON_EPS};
use catgraph_applied::mat::MatR;

/// Tsallis q-entropy `H_t(p) = (1 − Σ pᵢᵗ) / (t − 1)` for `t ≠ 1`.
///
/// At `t = 1` the limit is Shannon entropy `H₁(p) = -Σ pᵢ ln pᵢ`. Tsallis
/// 1988 / Havrda-Charvát 1967 introduce the parametric family; BV 2025 §3
/// uses it as the per-state language-model entropy in the closed-form
/// magnitude expression of Thm 3.10.
///
/// **Shannon special case.** When `|t − 1| < TSALLIS_SHANNON_EPS` (= `1e-6`),
/// the function returns `-Σ pᵢ ln pᵢ` directly to avoid catastrophic
/// cancellation in the `(1 − Σ pᵢᵗ) / (t − 1) ≈ 0/0` regime. Per Phase 6A
/// execution plan amend 5: the Cor 3.14 finite-difference step `h` MUST
/// satisfy `h > TSALLIS_SHANNON_EPS`; otherwise both `f(1+h)` and `f(1−h)`
/// evaluate the Shannon branch and the central difference collapses
/// identically to zero.
///
/// **Conventions.**
/// - Shannon branch: `0 · ln 0 = 0` by limit (terms with `pᵢ = 0` are skipped).
/// - Tsallis branch: `0^t = 0` for `t > 0`; `f64::powf` already returns `0.0`
///   for `0.0_f64.powf(t)` when `t > 0`, so zero-probability terms contribute
///   `0` to the sum without special handling.
/// - The function does NOT validate `Σ pᵢ = 1` — callers requiring a true
///   probability distribution must normalize beforehand. This keeps the
///   function compatible with random-vector proptest fixtures.
///
/// # Returns
///
/// `f64::NAN` only if `p` contains a NaN entry (propagates through `ln` and
/// `powf`). Otherwise a finite value (or `f64::INFINITY` if the Tsallis
/// branch divides by an extremely small `t − 1`, which the special case
/// short-circuits).
#[must_use]
pub fn tsallis_entropy(p: &[f64], t: f64) -> f64 {
    if (t - 1.0).abs() < TSALLIS_SHANNON_EPS {
        // Shannon branch: H₁(p) = -Σ pᵢ ln pᵢ, with `0 · ln 0 = 0` by limit.
        let mut sum = 0.0;
        for &pi in p {
            if pi > 0.0 {
                sum -= pi * pi.ln();
            }
        }
        sum
    } else {
        // Tsallis branch: H_t(p) = (1 − Σ pᵢᵗ) / (t − 1).
        // `0.0_f64.powf(t)` is `0.0` for `t > 0`; for the unusual `t < 0`
        // case, callers are responsible for excluding zero-probability terms.
        let sum_pow: f64 = p.iter().map(|&pi| pi.powf(t)).sum();
        (1.0 - sum_pow) / (t - 1.0)
    }
}

/// Möbius function of an enriched category, returned as an `n × n` matrix
/// of shape over `Q`, where `n = space.objects().count()`.
///
/// Per Leinster 2013 / Leinster-Shulman §2, the Möbius function `μ` is the
/// inverse of the zeta matrix `ζ` defined entrywise by
/// `ζ[i][j] = exp(-d(objects[i], objects[j]))` embedded into `Q` via
/// `Q::from(_: f64)`. Here `d` is the Lawvere distance carried by `space`.
///
/// **Bound: `Q: Ring + Div + From<f64>` — i.e. `Q` is a (commutative) field
/// for v0.1.0.** Gaussian elimination needs additive inverses (the `Ring`
/// bound, supplied by `Neg + Sub`) AND multiplicative inverses (the `Div`
/// bound, supplied by `Q / Q → Q`). Among the workspace's four concrete
/// rigs only [`crate::F64Rig`] satisfies all three; [`crate::BoolRig`],
/// [`crate::UnitInterval`], and [`crate::Tropical`] are excluded. The
/// chain-sum variant `mobius_function_via_chains<Q: Rig>` per Leinster-
/// Shulman's explicit formula is deferred to v0.2.0 — see crate root docs.
///
/// **Conversion `f64 → Q`.** The zeta matrix entries `exp(-d(i, j))` are
/// computed in `f64` then converted to `Q` via `Q::from(_)`. v0.1.0's only
/// `Ring + Div`-satisfying rig is `F64Rig`, which has the conversion
/// trivially.
///
/// # Errors
///
/// Returns [`CatgraphError::Composition`] when zeta is singular — i.e. when
/// Gaussian elimination cannot find a non-zero pivot in some column. No
/// Möbius function exists for that enriched category.
///
/// # Panics
///
/// Does not panic. Singular zeta returns `Err`; the implementation never
/// indexes out of bounds (matrix is `n × 2n` augmented and indices are
/// always `< n` or `< 2n` by construction).
pub fn mobius_function<Q>(
    space: &LawvereMetricSpace<NodeId>,
) -> Result<MatR<Q>, CatgraphError>
where
    Q: Ring + Div<Output = Q> + From<f64>,
{
    // Materialize the object list. `LawvereMetricSpace::objects()` (via the
    // `EnrichedCategory<Tropical>` impl) returns a `Box<dyn Iterator>` of
    // owned `NodeId`s, so we can collect directly.
    let objects: Vec<NodeId> = <LawvereMetricSpace<NodeId> as crate::EnrichedCategory<
        crate::Tropical,
    >>::objects(space)
    .collect();
    let n = objects.len();

    if n == 0 {
        // Empty category — Möbius function is the 0×0 matrix.
        return MatR::new(0, 0, Vec::new());
    }

    // Build the n × 2n augmented matrix [ζ | I] in `Vec<Vec<Q>>`. We do not
    // use `MatR` here because Gaussian elimination needs in-place row swaps
    // and arithmetic on individual entries — operations the immutable
    // `MatR` API does not expose.
    let mut aug: Vec<Vec<Q>> = (0..n)
        .map(|i| {
            let mut row: Vec<Q> = Vec::with_capacity(2 * n);
            // Left half: zeta[i][j] = exp(-d(objects[i], objects[j])).
            // Tropical(+∞) (unset distance) ⇒ exp(-∞) = 0; Tropical(0) ⇒
            // exp(0) = 1. f64::exp handles both correctly.
            for j in 0..n {
                let d = space.distance(&objects[i], &objects[j]);
                let zeta_ij: f64 = (-d.0).exp();
                row.push(Q::from(zeta_ij));
            }
            // Right half: identity.
            for j in 0..n {
                if i == j {
                    row.push(Q::one());
                } else {
                    row.push(Q::zero());
                }
            }
            row
        })
        .collect();

    // Gaussian-Jordan elimination with partial pivoting (find any non-zero
    // pivot — full pivoting is unnecessary for f64-backed rigs and rules
    // out the general `Q: Ring` future case).
    for col in 0..n {
        // Find a pivot row `pivot >= col` with non-zero entry in column `col`.
        let pivot = (col..n).find(|&r| !aug[r][col].is_zero());
        let Some(pivot) = pivot else {
            return Err(CatgraphError::Composition {
                message: format!("zeta matrix is singular at column {col}"),
            });
        };
        if pivot != col {
            aug.swap(col, pivot);
        }

        // Normalize pivot row: divide every entry in row `col` by the pivot.
        // Cloning the pivot value (rather than borrowing) sidesteps the
        // simultaneous-borrow conflict with `aug[col][k]`.
        let inv_pivot = Q::one() / aug[col][col].clone();
        // `needless_range_loop` would suggest iterating over the row, but we
        // need an indexed write back into `aug[col][k]`, so the index is the
        // primary loop variable, not just a counter.
        #[allow(clippy::needless_range_loop)]
        for k in 0..(2 * n) {
            let new_val = aug[col][k].clone() * inv_pivot.clone();
            aug[col][k] = new_val;
        }

        // Eliminate column `col` from every other row. We index into
        // BOTH `aug[col]` (read pivot row) and `aug[r]` (write target row)
        // inside the inner loop, so a flat `for k in 0..(2*n)` is the
        // simplest disambiguation; an iterator would require a `split_at_mut`
        // dance that doesn't improve readability.
        for r in 0..n {
            if r == col || aug[r][col].is_zero() {
                continue;
            }
            let factor = aug[r][col].clone();
            #[allow(clippy::needless_range_loop)]
            for k in 0..(2 * n) {
                let pivot_kth = aug[col][k].clone();
                let row_kth = aug[r][k].clone();
                aug[r][k] = row_kth - factor.clone() * pivot_kth;
            }
        }
    }

    // Extract the right half (now ζ⁻¹ = μ) into an n × n entries vector.
    let mu_entries: Vec<Vec<Q>> = aug
        .into_iter()
        .map(|row| row.into_iter().skip(n).collect())
        .collect();

    MatR::new(n, n, mu_entries)
}
