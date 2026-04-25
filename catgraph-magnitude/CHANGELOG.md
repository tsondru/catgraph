# Changelog

All notable changes to `catgraph-magnitude` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Phase 6A.2 `tsallis_entropy(p, t)` — Tsallis q-entropy
  `H_t(p) = (1 − Σ pᵢᵗ) / (t − 1)` with Shannon-recovery special case at
  `|t − 1| < TSALLIS_SHANNON_EPS = 1e-6`. The special-case branch returns
  `-Σ pᵢ ln pᵢ` directly, avoiding catastrophic cancellation in the `0/0`
  regime around `t = 1`. The Cor 3.14 finite-difference step `h` MUST stay
  above the threshold so both `f(1±h)` evaluate the Tsallis branch.
- Phase 6A.2 `mobius_function::<Q>(space)` — Möbius inversion `ζ · μ = I`
  via Gaussian elimination on an `n × 2n` augmented matrix `[ζ | I]`. Bound
  `Q: Ring + Div + From<f64>` — a (commutative) field for v0.1.0; only
  `F64Rig` qualifies among the workspace's four concrete rigs. Returns
  `Err(CatgraphError::Composition)` when zeta is singular. The chain-sum
  variant `mobius_function_via_chains<Q: Rig>` per Leinster-Shulman is
  deferred to v0.2.0.
- Tests: 4 proptest arms (Shannon recovery within ε threshold, Tsallis-to-
  Shannon limit on normalized distributions, μ·ζ=I on random Lawvere
  metric spaces) + 3 spot checks (basic Tsallis values, all-∞ singular
  zeta, all-zero singular zeta).
- Re-exports: `MatR` (from `catgraph-applied`), `CatgraphError` (from
  `catgraph::errors`).
- Phase 6A.0 scaffold: workspace member, `Cargo.toml`, `lib.rs` with module
  stubs + re-exports of the Tier 3 enrichment substrate from `catgraph-applied`
  v0.5.x (`Rig`, `UnitInterval`, `Tropical`, `F64Rig`, `BoolRig`,
  `EnrichedCategory`, `HomMap`, `LawvereMetricSpace`).
- `Ring` super-trait over `Rig` with blanket impl over `Neg + Sub`. Required
  by Möbius inversion in Phase 6A.2.
- `TSALLIS_SHANNON_EPS = 1e-6` public constant — Shannon special-case threshold
  for `tsallis_entropy` and lower bound for the Cor 3.14 finite-difference
  step.
- Phase 6A.1 `WeightedCospan<Λ, Q>` newtype wrapper over
  `catgraph::Cospan<Λ>` carrying per-edge weights in a rig `Q`. Public API:
  `from_cospan_uniform`, `from_cospan_with_weights`, `weight`, `set_weight`,
  `as_cospan`. Implied edges are the bipartite product
  `left_to_middle() × right_to_middle()` via the apex; absent entries return
  `Q::zero()`. Type aliases `ProbCospan<Λ>` (= `WeightedCospan<Λ,
  UnitInterval>`) and `TropCospan<Λ>` (= `WeightedCospan<Λ, Tropical>`).
  Specialized `into_metric_space` method on `WeightedCospan<Λ, UnitInterval>`
  lifts to a `LawvereMetricSpace<NodeId>` via the `-ln π` embedding
  (Lawvere 1973), reusing
  `LawvereMetricSpace::from_unit_interval`. Local `pub type NodeId = usize`
  one-to-one with the apex (middle) index. Tests: 2 proptest arms
  (round-trip + `set_weight` idempotence on `Q = F64Rig`) + 3 spot checks
  (metric-space embedding on `Q = UnitInterval`, absent-edge zero on
  `Q = Tropical`, per-pair `from_cospan_with_weights`).

[Unreleased]: https://github.com/tsondru/catgraph/compare/catgraph-applied-v0.5.2...HEAD
