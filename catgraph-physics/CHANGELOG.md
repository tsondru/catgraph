# Changelog

All notable changes to this crate are documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Performance candidates (bench-driven, no version target)

Deferred from Phase 3.1 rayon ride-along (2026-04-14). See `.claude/docs/ROADMAP.md` "Performance TODOs".

- `par_array_windows::<2>()` in `multiway::branchial::branchial_parallel_step_pairs` — per-pair work is cheap; bench on long foliations
- `par_array_windows::<2>()` in `hypergraph::evolution_cospan::to_cospan_chain` — per-pair work does a union-find pushout; benchable on long deterministic paths
- `walk_tree_prefix` / `walk_tree_postfix` in `multiway::evolution_graph` — compare against current recursive BFS / confluence-diamond enumeration
- rayon Producer/Consumer plumbing — reference design if `MultiwayEvolutionGraph` / `BranchialGraph` ever expose public parallel-iterator APIs

## [0.2.2] - 2026-04-19

Phase W.1 — WASM + edge-device support. Pass-through `parallel` feature
(this crate has no direct rayon call sites yet; the feature wires the
upstream `catgraph/parallel` toggle through so downstream builds with
`--no-default-features` see a single-threaded catgraph transitively).

### Added

- `[features] default = ["parallel"]` — `parallel = ["catgraph/parallel"]`.
- `examples/wasi_smoke_physics.rs` — small hypergraph construction smoke
  example.

### Changed

- `catgraph` dep now `default-features = false` so the `parallel` toggle
  propagates cleanly through this crate.

## [0.2.1] - 2026-04-17

### Changed

- Rustdoc framing pass (Phase 5.1): `src/multiway/evolution_graph.rs` module header extended with `## Time-step discretization as a functor F: C → D` and `## Per-step foliation selection` subsections. References Gorard 2023, Mamba state-space models, and BV 2025. No API changes.

## [0.2.0] - 2026-04-13

Branchial analysis toolkit — additive capabilities for `BranchialGraph`.

### Added

- `src/multiway/branchial_spectrum.rs`: `BranchialSpectrum` — graph Laplacian eigendecomposition via `SymmetricEigen`. Exposes algebraic connectivity (λ₂), spectral gap, Fiedler vector, connected-component count, spectral clustering (k-means on leading eigenvectors).
- `src/multiway/branchial_analysis.rs`: `to_petgraph()` conversion on `BranchialGraph`, plus `branchial_coloring` (greedy via rustworkx-core), `branchial_core_numbers` (k-core), `branchial_articulation_points`.
- Wasserstein DMatrix benchmark (`benches/wasserstein_bench.rs`) comparing `Vec<Vec<f64>>` vs `DMatrix<f64>` at sizes 10/50/100/200. Outcome: no performance delta — no refactor needed.

### Dependencies

- New: `nalgebra 0.34`, `nalgebra-sparse 0.11`, `petgraph 0.8`, `rustworkx-core 0.17`.
- New dev: `criterion 0.8`.

## [0.1.0] - 2026-04-12

### Added

- Initial release. Wolfram-physics extensions extracted from `catgraph` core (Phase 2):
  - `hypergraph/` — `Hypergraph`, `RewriteRule`, `HypergraphEvolution`, `HypergraphLattice` (gauge), categorical bridges (`rewrite_span.rs`, `evolution_cospan.rs`, `multiway_cospan.rs`).
  - `multiway/` — `MultiwayEvolutionGraph`, `BranchialGraph`, `OllivierRicciCurvature`, `wasserstein_1`.
- Gauge Wilson-loop fix: `record_transition(from, to, holonomy)` for explicit inter-site gauge links (was erroneously recording self-loops).
- Multiway APIs exposed for Phase 2.5 consumers in `irreducible`: `ConfluenceDiamond`, `confluence_diamonds()`, `parallel_independent_events(node_id)`, `events_commute(a, b)`.

[Unreleased]: https://github.com/tsondru/catgraph/compare/catgraph-physics-v0.2.2...HEAD
[0.2.2]: https://github.com/tsondru/catgraph/releases/tag/catgraph-physics-v0.2.2
[0.2.1]: https://github.com/tsondru/catgraph/releases/tag/catgraph-physics-v0.2.1
[0.2.0]: https://github.com/tsondru/catgraph/releases/tag/catgraph-physics-v0.2.0
[0.1.0]: https://github.com/tsondru/catgraph/releases/tag/catgraph-physics-v0.1.0
