//! Generic multiway (non-deterministic) computation infrastructure.
//!
//! Provides data structures and algorithms for branching computation systems
//! where multiple execution paths exist simultaneously. This includes:
//!
//! - [`MultiwayEvolutionGraph`]: Core graph for tracking branching state evolution
//! - [`run_multiway_bfs`]: Generic BFS explorer for any non-deterministic system
//! - [`BranchialGraph`]: Time-slice foliation (tensor product structure at each step)
//! - [`DiscreteCurvature`]: Trait for curvature backends on branchial graphs
//! - [`OllivierRicciCurvature`]: Ollivier-Ricci curvature via Wasserstein transport
//! - [`wasserstein_1`]: Transportation simplex W₁ optimal transport solver

pub mod wasserstein;
pub mod evolution_graph;
pub mod branchial;
pub mod curvature;
pub mod ollivier_ricci;

pub use evolution_graph::{
    run_multiway_bfs, BranchId, MergePoint, MultiwayCycle, MultiwayEdge, MultiwayEdgeKind,
    MultiwayEvolutionGraph, MultiwayNode, MultiwayNodeId, MultiwayStatistics,
};
pub use branchial::{
    branchial_to_parallel_intervals, extract_branchial_foliation, find_all_merge_points,
    BranchialGraph, BranchialStepStats, BranchialSummary,
};
pub use curvature::{CurvatureFoliation, DiscreteCurvature};
pub use ollivier_ricci::{OllivierFoliation, OllivierRicciCurvature};
pub use wasserstein::wasserstein_1;
