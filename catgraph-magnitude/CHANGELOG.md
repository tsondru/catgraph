# Changelog

All notable changes to `catgraph-magnitude` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-04-25

First publishable release. Anchored to BV 2025 (Bradley & Vigneaux,
*Magnitude of Language Models*, arXiv:2501.06662v2).

### Added

- Phase 6A.5 criterion bench (`benches/magnitude_bench.rs`) — three
  `mag_lm/<N>` benches (N = 10, 100, 1000) on acyclic forward-chain LMs at
  `t = 2.0`. Baseline median wall-clock (optimized, `--quick`):
  `mag_lm/10` ~30 µs, `mag_lm/100` ~11 ms, `mag_lm/1000` ~11 s.
  O(n³) Gaussian elimination dominates — 1000-state is the practical limit
  for the v0.1.0 dense-matrix Möbius implementation.

- Phase 6A.4 `examples/lm_magnitude.rs` — BV 2025 magnitude bounds
  demonstration on two contrasting LMs (deterministic 3-state, uniform
  5-state). Prints `Mag(tM)` at `t ∈ {0.5, 1.0, 2.0, 10.0, 1e6}` with
  Prop 3.10 closed-form comparison. Asserts four properties from BV 2025
  p.4 for `t ≥ 1`: (A) lower bound `≥ #T(⊥)`, (B) upper bound `≤ #ob(M)`,
  (C) monotone non-decreasing in `t`, (D) `Mag(1e6·M) ∈ [#T(⊥), #ob(M)]`.
  Verifies closed form = Möbius sum to `< 1e-9` at `t ∈ {0.5, 2.0, 10.0}`.
  Note: the `t → ∞` limit equals `#T(⊥)` only for fully-deterministic LMs
  (all-Dirac rows); for non-degenerate rows it is
  `#T(⊥) + #{non-degenerate non-terminal states}`.

- Phase 6A.4 `examples/tsallis_shannon.rs` — Tsallis-to-Shannon recovery
  (BV 2025 Rem 3.11) over 50 seeded random distributions (size 2–5) at
  `δt ∈ {1e-2, …, 1e-7}`. Asserts exact zero error within the
  `TSALLIS_SHANNON_EPS = 1e-6` special-case branch; asserts worst error
  `< 5e-3` at `δt = 1e-3`. Uses a minimal deterministic PCG-64-style LCG —
  same as `tests/lm_category.rs`. No `rand` dev-dep.

- Phase 6A.4 `examples/mock_coalition.rs` — 5-agent
  `WeightedCospan<&str, UnitInterval>` + 3-agent `LmCategory` diversity
  demo without any transport deps. Builds the 5-agent interaction graph
  (including a cycle), prints the Lawvere distance matrix, highlights
  `d(alice, bob) = -ln 0.7` and `d(alice, carol) = ∞` (no transitive
  closure in `into_metric_space`). Builds an acyclic 3-agent prefix-poset
  sub-coalition and prints four magnitude-derived indicators (`Mag(1.0)`,
  `Mag(2.0)`, `Mag(1e6)`, Shannon FD). Asserts BV 2025 p.4 bounds at
  `t = 2.0` and that `Mag(1e6·M) ∈ [#T(⊥), #ob(M)]`. Demonstrates the
  `WeightedCospan`/`LmCategory` API split (cyclic vs. acyclic view) before
  Phase 6B wires in `catgraph-coalition` transport.

- Phase 6A.4 `README.md` — replaced Phase 6A.0 stub with a v0.1.0-quality
  landing page. Includes: quickstart code snippet, two-point acceptance
  gate, full API surface table, algebraic + numerical scoping sections,
  three example descriptions, and roadmap.

- Phase 6A.4 rustdoc audit — fixed 3 pre-existing doc warnings: broken
  intra-doc link `catgraph::Cospan` (replaced with plain text), redundant
  explicit target in `ring.rs`, redundant explicit target in
  `lm_category.rs`. Zero doc warnings on `cargo doc`.

- Phase 6A.3 `magnitude::<Q>(space, t)` — magnitude
  `Mag(tM) = Σᵢⱼ μ_t[i][j]` of a Lawvere metric space at scale `t` via
  Möbius sum (BV 2025 §3.5, Eq 7). Builds a t-scaled copy of the input
  space, Möbius-inverts the resulting zeta matrix, and sums every entry.
  Same algebraic surface as `mobius_function`: `Q: Ring + Div + From<f64>`
  (only `F64Rig` qualifies in v0.1.0).

- Phase 6A.3 `LmCategory` — materialized language-model transition table
  per BV 2025 §3. Public API: `new`, `add_transition`, `mark_terminating`,
  `objects`, `terminating`, `transitions`, `magnitude(t)`. The `magnitude`
  method lifts the transition table into a `LawvereMetricSpace<NodeId>` via
  the prefix-extension semantics of BV 2025 §2.10–2.17: a forward BFS from
  each source state multiplies edge probabilities along every directed path,
  recording `d(x, y) = -ln π(y|x)` where `π(y|x)` is the product of
  intermediate transitions (Eq 6). Identity axiom `d(x, x) = 0` is
  enforced internally. The transition graph must be acyclic for magnitude to
  match Prop 3.10's closed form.

- Phase 6A.3 BV 2025 acceptance gate (`tests/bv_2025_acceptance.rs`):
  - **Prop 3.10 closed form** `Mag(tM) = (t−1)·Σ H_t(p_x) + #(T(⊥))`
    verified to `0e0` (exact `f64`) at `t ∈ {0.5, 1.5, 2.0, 5.0}` on a
    hand-computed 4-state LM (`A = {a}, N = 1`; states `⊥, ⊥a, ⊥†, ⊥a†`;
    `#T(⊥) = 2`).
  - **Rem 3.11 Shannon recovery** `d/dt Mag|_{t=1} = Σ H(p_x)` verified by
    central finite difference `(f(1+h) − f(1−h))/(2h)` with `h = 1e-4`.
    Observed residual `~6.46e-10`.

- Phase 6A.3 `LmCategory` unit tests (`tests/lm_category.rs`): empty-LM
  baseline (`Mag = n` for the identity zeta), round-trip on
  `add_transition` / `mark_terminating`, smoke test on the same 4-state
  tree fixture, and a BV 2025 Eq 4.3 bounds proptest
  (`#T(⊥) ≤ Mag(tM) ≤ #ob(M)` for `t ≥ 1`) on randomly generated
  forward-chain LMs of size 2–4.

- Phase 6A.2 `tsallis_entropy(p, t)` — Tsallis q-entropy
  `H_t(p) = (1 − Σ pᵢᵗ) / (t − 1)` with Shannon-recovery special case at
  `|t − 1| < TSALLIS_SHANNON_EPS = 1e-6`. The special-case branch returns
  `-Σ pᵢ ln pᵢ` directly, avoiding catastrophic cancellation in the `0/0`
  regime around `t = 1`. The Rem 3.11 finite-difference step `h` MUST stay
  above the threshold so both `f(1±h)` evaluate the Tsallis branch.

- Phase 6A.2 `mobius_function::<Q>(space)` — Möbius inversion `ζ · μ = I`
  via Gaussian elimination on an `n × 2n` augmented matrix `[ζ | I]`. Bound
  `Q: Ring + Div + From<f64>` — a (commutative) field for v0.1.0; only
  `F64Rig` qualifies among the workspace's four concrete rigs. Returns
  `Err(CatgraphError::Composition)` when zeta is singular. The chain-sum
  variant `mobius_function_via_chains<Q: Rig>` per Leinster-Shulman is
  deferred to v0.2.0.

- Tests: 4 proptest arms (Shannon recovery within ε threshold,
  Tsallis-to-Shannon limit on normalized distributions, μ·ζ=I on random
  Lawvere metric spaces) + 3 spot checks (basic Tsallis values, all-∞
  singular zeta, all-zero singular zeta).

- Re-exports: `MatR` (from `catgraph-applied`), `CatgraphError` (from
  `catgraph::errors`).

- Phase 6A.0 scaffold: workspace member, `Cargo.toml`, `lib.rs` with module
  stubs + re-exports of the Tier 3 enrichment substrate from
  `catgraph-applied` v0.5.x (`Rig`, `UnitInterval`, `Tropical`, `F64Rig`,
  `BoolRig`, `EnrichedCategory`, `HomMap`, `LawvereMetricSpace`).

- `Ring` super-trait over `Rig` with blanket impl over `Neg + Sub`. Required
  by Möbius inversion.

- `TSALLIS_SHANNON_EPS = 1e-6` public constant — Shannon special-case
  threshold for `tsallis_entropy` and lower bound for the Rem 3.11
  finite-difference step.

- Phase 6A.1 `WeightedCospan<Λ, Q>` newtype wrapper over
  `catgraph::Cospan<Λ>` carrying per-edge weights in a rig `Q`. Public API:
  `from_cospan_uniform`, `from_cospan_with_weights`, `weight`, `set_weight`,
  `as_cospan`. Absent entries return `Q::zero()`. Type aliases
  `ProbCospan<Λ>` (= `WeightedCospan<Λ, UnitInterval>`) and
  `TropCospan<Λ>` (= `WeightedCospan<Λ, Tropical>`). Specialized
  `into_metric_space` on `WeightedCospan<Λ, UnitInterval>` lifts to a
  `LawvereMetricSpace<NodeId>` via the `-ln π` embedding (Lawvere 1973).
  Tests: 2 proptest arms (round-trip + `set_weight` idempotence on
  `Q = F64Rig`) + 3 spot checks (metric-space embedding on `Q = UnitInterval`,
  absent-edge zero on `Q = Tropical`, per-pair `from_cospan_with_weights`).

### Acceptance gate

Both BV 2025 verifications pass at v0.1.0:

- **Prop 3.10 closed form** — `Mag(tM) = (t−1)·Σ H_t(p_x) + #(T(⊥))`
  verified to **0e0** (exact `f64`) on a 4-state hand-computed LM
  at `t ∈ {0.5, 1.5, 2.0, 5.0}`.
- **Rem 3.11 Shannon recovery** — `d/dt Mag|_{t=1} = Σ H(p_x)` by central
  finite difference (`h = 1e-4`) verified to **6.46e-10** on the same
  fixture.

### Numerical scoping

- `TSALLIS_SHANNON_EPS = 1e-6` — threshold below which `tsallis_entropy`
  returns `-Σ pᵢ ln pᵢ` directly to avoid catastrophic cancellation.
- Tsallis-Shannon worst-case recovery error: `0` (exact) at
  `δt < TSALLIS_SHANNON_EPS` (special-case branch); `< 5e-3` at
  `δt = 1e-3` (Tsallis branch).

### Performance baseline

`mag_lm/<N>` (criterion median wall-clock, optimized, `--quick`):

- `N = 10`: ~30 µs
- `N = 100`: ~11 ms
- `N = 1000`: ~11 s

### Dependencies

- `catgraph = "0.12"` (path dep during development; crates.io strips path on publish)
- `catgraph-applied = "0.5"` (requires v0.5.3+ for `F64Rig` ring + field ops)
- `num` (workspace dep)
- `proptest`, `criterion` (dev only)
- No tokio, no serde, no rayon

[Unreleased]: https://github.com/tsondru/catgraph/compare/catgraph-magnitude-v0.1.0...HEAD
[0.1.0]: https://github.com/tsondru/catgraph/releases/tag/catgraph-magnitude-v0.1.0
