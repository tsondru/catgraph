//! Category-theoretic graph structures with source/target (cospan) semantics.
//!
//! **catgraph** implements applied category theory for compositional systems,
//! focusing on Fong-Spivak-style string diagrams and cospans. Hyperedges connect
//! *source sets* to *target sets* (cospan semantics), as opposed to path-based
//! semantics where vertices are chained sequentially.
//!
//! ## Core abstractions
//!
//! - [`category`] — composition traits: [`HasIdentity`](category::HasIdentity),
//!   [`Composable`](category::Composable), [`ComposableMutating`](category::ComposableMutating)
//! - [`cospan`] — pushout composition over union-find (nearly linear time)
//! - [`span`] — pullback composition (dual of cospan) and [`Rel`](span::Rel) relation algebra
//! - [`named_cospan`] — port-labeled cospans for wiring-style composition
//! - [`monoidal`] — tensor product, symmetric braiding, generic layered morphisms
//!
//! ## String diagrams and operads
//!
//! - [`frobenius`] — Frobenius algebra generators, layered morphisms, DAG-based interpretation
//! - [`compact_closed`] — self-dual compact closed structure: cup/cap morphisms (Fong-Spivak §3.1)
//! - [`cospan_algebra`] — lax monoidal functors from cospans to sets (Fong-Spivak §2.1)
//! - [`hypergraph_category`] — hypergraph category trait with Frobenius generators (Fong-Spivak §2.3)
//! - [`hypergraph_functor`] — structure-preserving maps between hypergraph categories (Fong-Spivak §2.3)
//! - [`wiring_diagram`] — operadic substitution built on named cospans
//! - [`temperley_lieb`] — Temperley-Lieb / Brauer algebra via perfect matchings
//! - [`e1_operad`] / [`e2_operad`] — little intervals and little disks operads
//! - [`operadic`] — the `Operadic` trait for substitution
//!
//! ## Finite sets and combinatorics
//!
//! - [`finset`] — finite set morphisms, epi-mono factorization, permutations
//! - [`linear_combination`] — formal linear combinations over a coefficient ring
//!
//! ## Computational structures
//!
//! - [`interval`] / [`complexity`] / [`computation_state`] — discrete interval algebra
//!   for cobordism categories and complexity measures
//! - [`adjunction`] / [`bifunctor`] / [`coherence`] / [`stokes`] — functorial
//!   irreducibility framework (adjunctions, tensor products, coherence verification)
//! - [`trace`] — generic [`IrreducibilityTrace`](trace::IrreducibilityTrace) trait,
//!   [`analyze_trace`](trace::analyze_trace), repeat detection
//!
//! ## Petri nets
//!
//! - [`petri_net`] — place/transition nets with cospan bridge, firing, reachability,
//!   parallel and sequential composition

pub mod errors;
pub mod utils;
pub mod category;
pub mod monoidal;
pub mod operadic;
pub mod finset;
pub mod interval;
pub mod complexity;
pub mod computation_state;
pub mod adjunction;
pub mod bifunctor;
pub mod coherence;
pub mod stokes;
pub mod frobenius;
pub mod compact_closed;
pub mod cospan;
pub mod cospan_algebra;
pub mod hypergraph_category;
pub mod hypergraph_functor;
pub mod named_cospan;
pub mod span;
pub mod wiring_diagram;
pub mod linear_combination;
pub mod temperley_lieb;
pub mod e1_operad;
pub mod e2_operad;
pub mod petri_net;
pub mod trace;
pub mod hypergraph;
pub mod multiway;
