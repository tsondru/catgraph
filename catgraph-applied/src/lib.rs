//! Applied category theory extensions for catgraph.
//!
//! This crate packages modules that build on catgraph's Fong-Spivak 2019 core
//! (cospans, spans, Frobenius, hypergraph categories) but are **not** part of
//! the F&S 2019 paper's numbered content. It is the applied-CT complement to
//! the strict core crate.
//!
//! ## Modules
//!
//! - [`wiring_diagram`] — operadic substitution on named cospans
//! - [`petri_net`] — place/transition nets with cospan bridge
//! - [`temperley_lieb`] — Temperley-Lieb / Brauer algebra via perfect matchings
//! - [`linear_combination`] — formal linear combinations over a coefficient ring
//!   (used internally by `temperley_lieb`)
//! - [`e1_operad`] — little-intervals operad (E₁)
//! - [`e2_operad`] — little-disks operad (E₂)
//! - [`decorated_cospan`] — generic `DecoratedCospan<F>` over a `Decoration` functor
//!   (Fong–Spivak Def 6.75 + Thm 6.77; v0.3.0/v0.3.1)
//! - [`prop`] — symmetric strict monoidal categories with `Ob = ℕ` and the
//!   free prop `Free(G)` on a signature (F&S Def 5.2, Def 5.25; v0.4.0)
//! - [`operad_algebra`] — operad algebras `F : O → Set` with `CircAlgebra`
//!   (F&S Def 6.99, Ex 6.100; v0.4.0)
//! - [`operad_functor`] — functors between operads with the canonical
//!   `E₁ ↪ E₂` inclusion (F&S Rough Def 6.98; v0.4.0)
//!
//! ## Relationship to catgraph
//!
//! All modules depend on catgraph's public API:
//! - `Cospan`, `NamedCospan`, `Span`, `Rel` — pushout/pullback composition
//! - `Frobenius` generators — operadic composition of SMCs (Prop 3.8)
//! - `HypergraphCategory` trait — target of applied semantic functors
//! - `Operadic` trait — abstract interface for substitution
//! - `compact_closed` cup/cap — string-diagram rewriting (TL, wiring)
//!
//! See [`docs/SEVEN-SKETCHES-AUDIT.md`](https://github.com/tsondru/catgraph/blob/main/catgraph-applied/docs/SEVEN-SKETCHES-AUDIT.md)
//! for alignment with Fong & Spivak, *Seven Sketches in Compositionality*
//! (arXiv:1803.05316v3, 2018), Chapters 4–6.

/// Numerical epsilon for f32 geometric comparisons in operads.
pub(crate) const F32_EPSILON: f32 = 1e-6;

pub mod linear_combination;
pub mod mat;
#[cfg(feature = "f64-rig")]
pub mod mat_f64;
pub mod rig;
pub mod wiring_diagram;
pub mod temperley_lieb;
pub mod e1_operad;
pub mod e2_operad;
pub mod petri_net;
pub mod decorated_cospan;
pub mod prop;
pub mod operad_algebra;
pub mod operad_functor;
pub mod sfg;
pub mod sfg_to_mat;
pub mod graphical_linalg;
pub mod enriched;
