//! Hypergraph rewriting for compositional systems.
//!
//! Provides hypergraphs, DPO (Double-Pushout) rewrite rules, multiway
//! evolution tracking, gauge theory, and categorical span/cospan bridges.
//!
//! [`Hyperedge`] is an ordered vertex sequence generalizing graph edges.
//! [`Hypergraph`] stores vertices and hyperedges with pattern matching.

pub mod evolution;
pub mod gauge;
pub mod hyperedge;
#[allow(clippy::module_inception)]
pub mod hypergraph;
pub mod rewrite_rule;

pub use evolution::{
    CausalInvarianceResult, HypergraphEvolution, HypergraphNode, HypergraphStep, WilsonLoop,
};
pub use gauge::{
    plaquette_action, total_action, GaugeGroup, HypergraphLattice, HypergraphRewriteGroup,
};
pub use hyperedge::Hyperedge;
pub use hypergraph::Hypergraph;
pub use rewrite_rule::{RewriteMatch, RewriteRule, RewriteSpan};
