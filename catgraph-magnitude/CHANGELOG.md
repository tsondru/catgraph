# Changelog

All notable changes to `catgraph-magnitude` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Phase 6A.0 scaffold: workspace member, `Cargo.toml`, `lib.rs` with module
  stubs + re-exports of the Tier 3 enrichment substrate from `catgraph-applied`
  v0.5.x (`Rig`, `UnitInterval`, `Tropical`, `F64Rig`, `BoolRig`,
  `EnrichedCategory`, `HomMap`, `LawvereMetricSpace`).
- `Ring` super-trait over `Rig` with blanket impl over `Neg + Sub`. Required
  by Möbius inversion in Phase 6A.2.
- `TSALLIS_SHANNON_EPS = 1e-6` public constant — Shannon special-case threshold
  for `tsallis_entropy` and lower bound for the Cor 3.14 finite-difference
  step.

[Unreleased]: https://github.com/tsondru/catgraph/compare/catgraph-applied-v0.5.2...HEAD
