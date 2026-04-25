//! # catgraph-magnitude
//!
//! Magnitude of enriched categories for the catgraph workspace. Anchored to
//! Bradley & Vigneaux, *Magnitude of Language Models* (2025).
//!
//! ## Scope (v0.1.0)
//!
//! - [`WeightedCospan<Λ, Q>`](weighted_cospan::WeightedCospan) — newtype over
//!   [`catgraph::Cospan`] with per-edge weights in a rig `Q`.
//! - [`tsallis_entropy`](magnitude::tsallis_entropy) — `H_t(p) = (1 − Σ pᵢᵗ)/(t−1)`,
//!   special-cased to Shannon at `|t−1| < TSALLIS_SHANNON_EPS`.
//! - [`mobius_function`](magnitude::mobius_function) — Möbius inversion
//!   `ζ · μ = I` over a [`Ring`] (additive-inverses required).
//! - [`magnitude`](magnitude::magnitude) — magnitude via Möbius sum.
//! - [`LmCategory`](lm_category::LmCategory) — materialized language-model
//!   transition table with `Mag(tM)` per BV 2025 Thm 3.10.
//!
//! ## Substrate
//!
//! Re-exports the Tier 3 enrichment infrastructure from `catgraph-applied`
//! v0.5.x — [`Rig`], [`UnitInterval`], [`Tropical`], [`F64Rig`], [`BoolRig`],
//! [`EnrichedCategory`], [`HomMap`], [`LawvereMetricSpace`].
//!
//! ## v0.1.0 algebraic scoping
//!
//! Möbius inversion via Gaussian elimination on `Matrix<Q>` requires `Q` to
//! have additive inverses — i.e. a **ring**, not merely a rig. v0.1.0 exposes
//! a thin [`Ring`] super-trait over [`Rig`] and restricts
//! [`mobius_function`](magnitude::mobius_function) /
//! [`magnitude`](magnitude::magnitude) to `Q: Ring`. `F64Rig` satisfies
//! `Ring`; `BoolRig`, `UnitInterval`, `Tropical` do not. A chain-sum
//! `mobius_function_via_chains<Q: Rig>` per Leinster-Shulman §2 is deferred
//! to v0.2.0.
//!
//! ## Numerical scoping
//!
//! [`TSALLIS_SHANNON_EPS`] = `1e-6` is the threshold below which
//! [`tsallis_entropy`](magnitude::tsallis_entropy) returns the Shannon limit
//! `-Σ pᵢ ln pᵢ` directly, avoiding catastrophic cancellation in the
//! `O(1e-9) / 1e-9` regime around `t = 1`. The Cor 3.14 finite-difference
//! step `h` must satisfy `h > TSALLIS_SHANNON_EPS` so both `f(1±h)` evaluate
//! the Tsallis branch (recommended `h = 1e-4`, ~2 decimal margin above the
//! threshold while staying near `f64`'s `ε^(1/3) ≈ 6e-6` truncation+roundoff
//! optimum).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod ring;

// Phase 6A.1 / 6A.2 / 6A.3 module stubs — populated in subsequent commits.
pub mod weighted_cospan;
pub mod magnitude;
pub mod lm_category;

// Re-exports of the Tier 3 enrichment substrate from catgraph-applied.
pub use catgraph_applied::rig::{BoolRig, F64Rig, Rig, Tropical, UnitInterval};
pub use catgraph_applied::enriched::{EnrichedCategory, HomMap};
pub use catgraph_applied::lawvere_metric::LawvereMetricSpace;
pub use catgraph_applied::mat::MatR;
pub use catgraph::errors::CatgraphError;

pub use ring::Ring;

/// Threshold for the Shannon special case in
/// [`tsallis_entropy`](magnitude::tsallis_entropy). For `|t − 1| < ε`, the
/// function returns `-Σ pᵢ ln pᵢ` directly, avoiding catastrophic cancellation
/// in the `(1 − Σ pᵢᵗ)/(t − 1)` ≈ `0/0` regime.
///
/// The Cor 3.14 finite-difference step `h` MUST satisfy
/// `h > TSALLIS_SHANNON_EPS`, otherwise both `f(1+h)` and `f(1−h)` evaluate
/// the Shannon branch and the central difference collapses to identically
/// zero. Recommended `h = 1e-4` — see crate-level docs.
pub const TSALLIS_SHANNON_EPS: f64 = 1e-6;
