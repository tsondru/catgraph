//! Hypergraph rewriting for compositional systems.
//!
//! Provides hypergraphs, DPO (Double-Pushout) rewrite rules, multiway
//! evolution tracking, gauge theory, and categorical span/cospan bridges.
//!
//! [`Hyperedge`] is an ordered vertex sequence generalizing graph edges.
//! [`Hypergraph`] stores vertices and hyperedges with pattern matching.

pub mod hyperedge;
#[allow(clippy::module_inception)]
pub mod hypergraph;

pub use hyperedge::Hyperedge;
pub use hypergraph::Hypergraph;
