# Changelog

All notable changes to this crate are documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

No in-flight work.

### Performance candidates (bench-driven, no version target)

Deferred from Phase 3.1 rayon ride-along (2026-04-14). See `.claude/docs/ROADMAP.md` "Performance TODOs".

- `par_array_windows::<2>()` at `catgraph-physics::branchial_parallel_step_pairs` + `evolution_cospan::to_cospan_chain` — bench-driven
- `walk_tree_prefix` / `walk_tree_postfix` for multiway BFS / confluence-diamond enumeration
- `fold_chunks` / `fold_chunks_with` for Phase 6 magnitude per-partition accumulation
- rayon Producer/Consumer plumbing if public parallel-iterator APIs land on `MultiwayEvolutionGraph` / `BranchialGraph`

## [0.5.0] - 2026-04-21

Tier 3 applied-CT closures — F&S *Seven Sketches* Chapter 5 main content:
the prop presentation machinery, functorial semantics `S: SFG_R → Mat(R)`,
and the 16-equation Thm 5.60 presentation of Mat(R). Also closes §6.3 Ex 6.64
(Corel as `HypergraphCategory`) via catgraph v0.12.0 core.

### Added

- `src/rig.rs` — `Rig` trait (F&S Def 5.36) as a blanket impl over
  `num_traits::{Zero, One}` + `Add` + `Mul`. 4 concrete instances:
  `BoolRig` (∨, ∧), `UnitInterval` ([0,1] Viterbi semiring; BTV 2021
  enrichment base), `Tropical` ([0,∞], min, +, ∞, 0; Lawvere metric / magnitude
  homology base), `F64Rig` (real demo rig). `BaseChange<UnitInterval>` for
  `Tropical` via `d = −ln π`. `verify_rig_axioms` runtime check returning
  `CatgraphError::RigAxiomViolation`.
- `src/prop/presentation.rs` — `Presentation<G>` (F&S Def 5.33) with
  `add_equation`, `normalize`, `eq_mod`, `with_depth`. 8-rule SMC canonical
  form applied first (closes Def 5.30 PARTIAL gap); user equations applied
  left-to-right. Bounded-depth rewriting (default 32); Knuth-Bendix
  completion is v0.5.1 work.
- `src/sfg.rs` — `SignalFlowGraph<R>` (F&S Def 5.45). 5 primitive generators
  from Eq 5.52: Copy 1→2, Discard 1→0, Add 2→1, Zero 0→1, Scalar(r) 1→1.
  Derived `copy_n` / `discard_n` as iterated compositions.
- `src/mat.rs` — `MatR<R>` matrix prop (F&S Def 5.50) over any `Rig` R,
  backed by `Vec<Vec<R>>`. F&S convention: morphism `m → n` is `m × n`.
  `Composable`, `Monoidal`, `SymmetricMonoidalMorphism` + `block_diagonal`
  tensor. Works for Tropical, Boolean, and UnitInterval without nalgebra.
- `src/sfg_to_mat.rs` — `sfg_to_mat` functor `S: SFG_R → Mat(R)` (F&S
  Thm 5.53). Structural recursion over `PropExpr<SfgGenerator<R>>`; generator
  matrix table matches Eq 5.52 exactly. Functoriality on all 4 rigs verified
  via 13 integration tests.
- `src/graphical_linalg.rs` — `matr_presentation<R>` builds the 16 equations
  from F&S Thm 5.60 p.170 (Groups A cocomonoid, B monoid, C bialgebra,
  D scalar). `verify_sfg_to_mat_is_full_and_faithful<R>` enumeration harness.
- `src/mat_f64.rs` (feature `f64-rig`, opt-in) — nalgebra bridge for
  `MatR<F64Rig>`: `mat_to_nalgebra` / `mat_from_nalgebra` roundtrip,
  `determinant`, `try_inverse`.
- 9 new integration test files + 2 runnable examples (`rig_showcase`,
  `sfg_to_mat`).

### Changed

- `src/prop.rs` → `src/prop/mod.rs` (directory module) to host the new
  `presentation` submodule. API unchanged; all v0.4.0 prop tests continue
  to pass.
- `PropSignature: Eq` relaxed to `PropSignature: PartialEq` with matching
  `#[derive(PartialEq)]` on `PropExpr`. Required to use f64-backed rigs
  (`UnitInterval`, `F64Rig`, `Tropical`) as `Scalar(R)` generator payloads
  inside `SfgGenerator<R>`. Strict weakening — all existing impls that
  required `Eq` still compile.
- catgraph dep bumped to v0.12.0 (for `Corel<Lambda>` + new error variants
  `Presentation`, `SfgFunctor`, `RigAxiomViolation`).

### Features

- `f64-rig` (opt-in, off by default) — enables the `mat_f64` module and adds
  a transitive `nalgebra` dep. Non-f64 rig users skip nalgebra entirely.

### Known limitations

- **Thm 5.60 faithfulness enumeration tests `#[ignore]`'d.** The 12
  `thm_5_60_faithful_*` tests in `tests/graphical_linalg.rs` are marked
  `#[ignore]` with documented reason: `Presentation::normalize` uses bounded
  structural rewriting without Knuth-Bendix completion; the D-group scalar
  equations heavily overlap and produce false-negative equivalence-class
  splits. The equation set itself is correct — all 16 F&S p.170 equations
  construct cleanly — and soundness smoke tests pass. Audit §5.4 Thm 5.60
  is **PARTIAL** in v0.5.0. **v0.5.1 will add KB completion and re-enable
  the faithfulness tests.**

### Requires

- catgraph v0.12.0 (new error variants + `Corel<Lambda>`).

## [0.4.0] - 2026-04-20

Tier 2 applied-CT gap closures from `docs/SEVEN-SKETCHES-AUDIT.md`. Three
new modules anchored to F&S *Seven Sketches in Compositionality*
§5.2 and §6.5; no changes to existing public APIs.

### Added

- `prop` module (Def 5.2, Def 5.25). `PropSignature` trait for generator
  arities; arity-tracked `PropExpr<G>` expression tree; smart constructors
  `Free::{identity, braid, generator, compose, tensor}` with
  composition-arity validation. Implements `Composable<Vec<()>>`,
  `HasIdentity<Vec<()>>`, `Monoidal`, and `SymmetricMonoidalMorphism<()>`.
  Equality is structural — the SMC quotient (interchange law, unitors,
  braiding naturality) is deferred to v0.5.0 alongside the Tier 3
  presentation / equations type (Def 5.33).
- `operad_algebra` module (Def 6.99). Single-sorted `OperadAlgebra<O, Input>`
  trait `F : O → Set` generic over any `Operadic<Input>` type. Concrete
  `CircAlgebra` implementing F&S Ex 6.100 for `WiringDiagram` via
  outer-port counts; `check_substitution_preserved` helper witnessing
  `evaluate(op ∘_i inner, inputs) == evaluate(op, inputs)` for algebras
  whose evaluator discards inputs.
- `operad_functor` module (Rough Def 6.98). Generic `OperadFunctor<O1, O2, Input>`
  trait. Concrete `E1ToE2` packaging the canonical little-intervals-into-
  little-disks inclusion (via upstream `E2::from_e1_config`) with a
  `start_name` offset so the two branches of `F(o ∘_i q) = F(o) ∘_i F(q)`
  can share a substitution without colliding on E2's unique-name
  invariant. Literal geometric functoriality is verified by
  `E1ToE2::check_substitution_preserved` (canonicalising each side's disks
  by centre-x and comparing within `f32` tolerance); a generic arity-level
  shadow `check_substitution_preserved` covers any `OperadFunctor`.
- Public accessors `E1::arity`, `E1::sub_intervals`, `E2::arity_of`,
  `E2::sub_circles`; `#[derive(Clone)]` on `E1` and `E2<Name: Clone>`.
  Additive and non-breaking.
- Examples: `examples/free_prop.rs`, `examples/operad_algebra_circ.rs`,
  `examples/operad_functor_e1_to_e2.rs`.
- Tests: `tests/prop.rs` (11 tests), `tests/operad_algebra.rs` (3 tests),
  `tests/operad_functor.rs` (4 tests).

### Requires

- catgraph v0.11.4 (unchanged from v0.3.3).

## [0.3.3] - 2026-04-19

Phase W.1 — WASM + edge-device support. Wires the `parallel` feature
through all four `CondIterator` call sites; compiles clean against
`wasm32-wasip1-threads` and `wasm32-wasip1 --no-default-features`.

### Added

- `[features] default = ["parallel"]` — `parallel = ["dep:rayon",
  "dep:rayon-cond", "catgraph/parallel"]`.
- `examples/wasi_smoke_applied.rs` — representative `LinearCombination`
  multiplication example.

### Changed

- `rayon` and `rayon-cond` are now optional dependencies gated by the
  `parallel` feature.
- `catgraph` dep is `default-features = false` so the `parallel` toggle
  propagates.
- `src/linear_combination.rs::Mul::mul` and `::linear_combine`:
  `CondIterator::new(...).map(...).collect()` gated with
  `#[cfg(feature = "parallel")]`; plain `into_iter().map(...).collect()`
  fallback when off. Shared closure extracted so both arms use identical
  per-term logic.
- `src/temperley_lieb.rs::BrauerMorphism::non_crossing`: both `source`
  and `target` crossing checks use `CondIterator::new(...).any(...)`
  under `#[cfg(feature = "parallel")]`; plain `.into_iter().any(...)`
  fallback when off. Shared `has_crossing` predicate extracted once.
- `tests/rayon_equivalence.rs`: the three direct `CondIterator`
  arm-equivalence tests are gated behind `#[cfg(feature = "parallel")]`
  (they test the rayon_cond dep, which is only in the graph when the
  feature is on).

### Notes

- Native test count: 900 with default features, 897 with
  `--no-default-features` (the 3 gated tests).

## [0.3.2] - 2026-04-19

Phase W.0 pre-WASM rayon consolidation. Internal-only — no public API change. See [`.claude/plans/i-realize-i-need-wise-stonebraker.md`](../.claude/plans/i-realize-i-need-wise-stonebraker.md) for the WASM roadmap that motivates this patch.

### Changed

- `linear_combination::Mul::mul` and `linear_combination::LinearCombination::linear_combine` now use `rayon_cond::CondIterator` to unify the parallel/sequential branches at the two `HashMap` `into_par_iter()` call sites. Functional behavior unchanged — `PARALLEL_MUL_THRESHOLD = 32` still gates the parallel path.
- `temperley_lieb::BrauerMorphism::non_crossing` now uses `rayon_cond::CondIterator` to unify the parallel/sequential branches at the two `par_bridge()` call sites. Functional behavior unchanged — `PARALLEL_COMBINATIONS_THRESHOLD = 8` still gates the parallel path.

### Added

- `rayon-cond = "0.4"` as a direct dependency (previously pulled transitively via `rustworkx-core`).
- `tests/rayon_equivalence.rs` extended to exercise both `CondIterator::Parallel` and `CondIterator::Serial` arms at each migrated site, asserting algebraic-law determinism across the toggle.

### Why this shape

The previous if/else-over-threshold pattern duplicated the iteration body. `rayon_cond::CondIterator` is the canonical rustworkx-core idiom (see [`rustworkx-core/src/centrality.rs`](https://github.com/Qiskit/rustworkx/blob/main/rustworkx-core/src/centrality.rs)) for compile/runtime parallel↔sequential toggling, and it's the right pattern for Phase W.1's `parallel` feature flag — a single `#[cfg(feature = "parallel")]` gate replaces cfg-gating two parallel branches.

## [0.3.1] - 2026-04-18

Tier 1.1 follow-ups flagged during v0.3.0 work.

### Added

- `DecoratedCospan::compose` now invokes `D::pushforward` through the pushout quotient (realizes F&S Def 6.75 / Thm 6.77 for decorations whose apex data references apex indices).
- Direct `PetriNet::permute_side` implementation via in-place permutation of the transition sequence — replaces the decoration-bridge impl that discarded boundary permutations on the return trip.
- `Transition::relabel` arc deduplication: when the quotient collapses distinct places onto the same target, arcs merge with summed `Decimal` multiplicities. Pre- and post-arcs dedup independently (self-loops preserved). Canonical ascending-by-place sort.
- `examples/petri_net_braiding.rs` — direct `permute_side` demo.
- `tests/decorated_cospan.rs` — 3 integration tests covering Circuit EdgeSet series composition, `Trivial` pushforward unit, `PetriDecoration` regression safety.
- `tests/petri_net.rs` — 8 new tests (4 braiding + 4 arc-dedup).

### Changed

- `examples/decorated_cospan_circuit.rs` extended with series composition; `NOTE:` caveat block removed.
- `SEVEN-SKETCHES-AUDIT.md` Ex 6.79–6.86 row upgraded from PARTIAL to DONE; headline recomputed (9 DONE / 3 PARTIAL / 12 MISSING / 17 N/A / 15 IN CORE of 56 items).

### Requires

- catgraph v0.11.3 for `Cospan::compose_with_quotient`.

## [0.3.0] - 2026-04-17

Tier 1 gap closures (from v0.2.0 audit).

### Added

- Generic `DecoratedCospan<Lambda, D>` + `Decoration` trait — realizes F&S Def 6.75 (decorated cospans) and Thm 6.77 (decorated cospan category is a hypergraph category).
- `PetriDecoration<Lambda>` marker type bridging `PetriNet` to the generic `DecoratedCospan` machinery.
- `HypergraphCategory<Lambda>` impl for both `DecoratedCospan<Lambda, D>` (generic) and `PetriNet<Lambda>` (specialized).
- `examples/decorated_cospan_circuit.rs` — Circuit EdgeSet example.
- `Trivial` decoration as an uninformative starting example.

### Known limitations (closed in 0.3.1)

- `DecoratedCospan::compose` did not yet invoke `D::pushforward` (required upstream `Cospan::compose_with_quotient`).
- `PetriNet::permute_side` delegated to the decoration bridge, which discarded leg permutations.
- `Transition::relabel` produced duplicate `(place, weight)` entries when the quotient collapsed places.

## [0.2.0] - 2026-04-17

### Added

- `docs/SEVEN-SKETCHES-AUDIT.md` — section-by-section coverage audit against Fong & Spivak, *Seven Sketches in Compositionality* (arXiv:1803.05316v3, 2018). 56 items tracked across Chapters 4–6.
- Cross-reconciliation with `catgraph/docs/FONG-SPIVAK-AUDIT.md`.

## [0.1.0] - 2026-04-14

### Added

- Initial release. Applied-CT modules extracted from `catgraph` core as part of the v0.11.0 slim-baseline refactor:
  - `linear_combination.rs` — formal linear combinations over a coefficient ring (R-module `R[T]`).
  - `wiring_diagram.rs` — operadic substitution on named cospans (F&S §6.5 Ex 6.94 Cospan operad).
  - `petri_net.rs` — place/transition nets, firing, reachability, parallel/sequential composition, cospan bridge.
  - `temperley_lieb.rs` — Temperley-Lieb / Brauer algebra via perfect matchings, Jones relations, dagger.
  - `e1_operad.rs` — little-intervals operad (E₁).
  - `e2_operad.rs` — little-disks operad (E₂).
- Criterion bench `rayon_thresholds`.

[Unreleased]: https://github.com/tsondru/catgraph/compare/catgraph-applied-v0.5.0...HEAD
[0.5.0]: https://github.com/tsondru/catgraph/compare/catgraph-applied-v0.4.0...catgraph-applied-v0.5.0
[0.4.0]: https://github.com/tsondru/catgraph/releases/tag/catgraph-applied-v0.4.0
[0.3.3]: https://github.com/tsondru/catgraph/releases/tag/catgraph-applied-v0.3.3
[0.3.2]: https://github.com/tsondru/catgraph/releases/tag/catgraph-applied-v0.3.2
[0.3.1]: https://github.com/tsondru/catgraph/releases/tag/catgraph-applied-v0.3.1
[0.3.0]: https://github.com/tsondru/catgraph/releases/tag/catgraph-applied-v0.3.0
[0.2.0]: https://github.com/tsondru/catgraph/releases/tag/catgraph-applied-v0.2.0
[0.1.0]: https://github.com/tsondru/catgraph/releases/tag/catgraph-applied-v0.1.0
