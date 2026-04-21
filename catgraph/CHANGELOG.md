# Changelog

All notable changes to this crate are documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

No in-flight work.

## [0.12.0] - 2026-04-21

Adds `Corel<Lambda>` — the dual of `Rel<Lambda>`, realizing F&S 2018 (Seven
Sketches) Example 6.64: Corel as a hypergraph category. Unblocks the Phase 6
catgraph-magnitude roadmap, which uses `WeightedCorel<Λ, Q>` as the enriched
counterpart for LM-enriched categories (BV 2025).

### Added

- `src/corel.rs` — `Corel<Lambda>` newtype over `Cospan<Lambda>`:
  - Constructors `new` (validates joint surjectivity) + `new_unchecked`.
  - Accessors `as_cospan`, `equivalence_classes`.
  - Predicates `merges`, `refines`, `coarsest_common_refinement`,
    `is_identity_partition`.
  - Trait impls: `HasIdentity`, `Composable<Vec<Lambda>>`, `Monoidal`,
    `MonoidalMorphism`, `SymmetricMonoidalMorphism`, and
    `HypergraphCategory<Lambda>` (F&S 2018 Ex 6.64).
- `Cospan::is_jointly_surjective()` — inherent helper used by `Corel::new`.
- `CatgraphError::Corel { message: String }` — new error variant.
- `tests/corel.rs` — 9 integration tests for constructors, predicates,
  composition.
- `tests/corel_hypergraph_category.rs` — 8 tests verifying F&S 2018 Ex 6.64.
- `tests/common/mod.rs` — `corel_eq` and `assert_corel_eq` helpers.
- `tests/rayon_equivalence.rs` — determinism test for
  `coarsest_common_refinement`.
- `examples/corel.rs` — runnable example.

### Scope notes

- `Corel` lives in catgraph core because it is a §2/§3 F&S 2019 item (dual
  of `Rel`) and reuses the core's compact-closed / Frobenius machinery
  directly. No separate crate.
- This release is **additive**: no API changes on `Cospan`, `Span`, `Rel`,
  `Frobenius`, `CospanAlgebra`, or `HypergraphFunctor`. Downstream crates
  (`catgraph-physics`, `catgraph-applied`, `catgraph-surreal`, `irreducible`)
  can bump to `v0.12.0` opportunistically without code changes.

## [0.11.4] - 2026-04-19

Phase W.1 — WASM + edge-device support. Internal-only: adds a `parallel`
feature flag (default-on) and gates the two rayon call sites so the crate
compiles clean against `wasm32-wasip1-threads` and `wasm32-wasip1`
(single-threaded) with `--no-default-features`. See
[`.claude/plans/i-realize-i-need-wise-stonebraker.md`](../.claude/plans/i-realize-i-need-wise-stonebraker.md).

### Added

- `[features] default = ["parallel"]` — `parallel = ["dep:rayon"]`.
  Native default-on (no behavior change for native users); disable with
  `--no-default-features` on single-threaded WASM hosts.
- `examples/wasi_smoke_core.rs` — representative cospan composition
  example, verifies the core API round-trips under WASI (build: `cargo
  build --lib --target wasm32-wasip1-threads -p catgraph`).

### Changed

- `rayon` is now an optional dependency gated by the `parallel` feature.
- `src/named_cospan.rs::find_nodes_by_name_predicate` parallel branch
  (lines 329-343) gated with `#[cfg(feature = "parallel")]`; plain
  `iter()` fallback when the feature is off.
- `src/frobenius/operations.rs::FrobeniusLayer::hflip` parallel branch
  gated with `#[cfg(feature = "parallel")]`; plain `iter_mut()` fallback
  when off.
- Top-level `rust-toolchain.toml` added, pinning stable + targets
  `wasm32-wasip1` + `wasm32-wasip1-threads`.
- Top-level `.cargo/config.toml` added with placeholder `rustflags = []`
  entries for both WASI sub-targets (documentation point for host-specific
  tweaks).

### Known cargo quirks

- `cargo build --example ... --target wasm32-wasip1-threads` fails on the
  transitive `proptest → rusty-fork → wait-timeout` dev-dep because
  `wait-timeout` doesn't support `wasm32-*`. Cargo resolves dev-deps for
  any `-p catgraph` build even when you only ask for an example.
  Workaround: use `cargo build --lib --target ...` for WASM library
  verification, or temporarily comment out `proptest` in
  `[dev-dependencies]` when you need to build the example artifact.

## [0.11.3] - 2026-04-18

### Added

- `Cospan::compose_with_quotient(&self, other: &Self) -> Result<(Self, Vec<usize>), CatgraphError>` — additive public method exposing the union-find pushout quotient map. Indexing convention: positions `0..self.middle.len()` map `self`'s middle indices; next slice maps `other`'s middle indices; both map into `0..composed.middle.len()`.

### Changed

- `Composable::compose` on `Cospan` is now a thin wrapper around `compose_with_quotient` (behavior unchanged; quotient discarded for `compose` callers).

### Notes

- The quotient is consumed by `catgraph-applied::DecoratedCospan::compose` to realize the `F(q) ∘ combine(d₁, d₂)` formula of Fong-Spivak Thm 6.77 (Def 6.75).

## [0.11.2] - 2026-04-17

### Added

- Explicit tests for Thm 6.55 spider theorem (`tests/spider_theorem.rs`): shape equality between connected Frobenius diagrams and canonical spiders produced by `special_frobenius_morphism(m, n, z)`.

## [0.11.1] - 2026-04-17

### Changed

- Phase 5 cross-link: `FONG-SPIVAK-AUDIT.md` "Reconciliation" section cross-references `catgraph-applied/docs/SEVEN-SKETCHES-AUDIT.md`.

## [0.11.0] - 2026-04-14

### Changed

- **Slim F&S baseline release.** Non-F&S applied modules relocated to sibling workspace crates: Petri nets, E_n operads, Temperley-Lieb, wiring diagrams, linear combinations moved to `catgraph-applied`; hypergraph DPO rewriting, multiway evolution, gauge theory, branchial spectral analysis moved to `catgraph-physics`.

### Removed

- `src/petri_net.rs`, `src/wiring_diagram.rs`, `src/e1_operad.rs`, `src/e2_operad.rs`, `src/temperley_lieb.rs`, `src/linear_combination.rs` (→ catgraph-applied v0.1.0).
- `src/hypergraph/`, `src/multiway/` (→ catgraph-physics v0.1.0, Phase 2).

### Notes

- Workspace members now: `catgraph` (this), `catgraph-physics`, `catgraph-applied`.
- ~490 tests after slimming (down from 630).

## [0.10.6] - 2026-04-12

### Changed

- Phase 2: hypergraph + multiway subsystems extracted to new workspace member `catgraph-physics` v0.1.0. Gauge Wilson-loop fix (`record_transition(from, to, holonomy)` for explicit inter-site links). Multiway APIs for Phase 2.5 (confluence diamonds, parallel-independent events, causal commutation) pinned as public for downstream consumers.

## [0.10.5] - 2026-04-11

### Changed

- Phase 1: Group 7 modules (adjunction, bifunctor, coherence, complexity, computation_state, interval, stokes, trace) moved back to `irreducible`. `Cospan::compose_chain` helper added; `trace_path_to_root` made public; interval-typed bridges replaced with raw `(usize, usize)` pair APIs in multiway.

## [0.10.4] - 2026-04-11

### Added

- Phase 0.5: closed all 5 F&S audit gaps — Lemma 4.3 (`functor_induced_algebra_map`), Lemma 4.9 (`functor_from_algebra_morphism`), Prop 3.4 (name recovery test), Prop 4.6 (Part initiality test), `compose_names_direct` matching Prop 3.3 literal formula.

### Fixed

- `two_layer_simplify` Rule 2 bug permitting `permutation_automatic` to come out of `#[ignore]` and gating 6 production unwraps.

## [0.10.3] - 2026-04-10

### Changed

- Phase 0.0: workspace restructure. SurrealDB persistence extracted to sibling repo `catgraph-surreal` v0.7.0.

## [0.10.0] - 2026-04-08

### Added

- Fong-Spivak §2–3 modules: `cospan_algebra.rs`, `hypergraph_category.rs`, `hypergraph_functor.rs`, `compact_closed.rs`, `equivalence.rs`. Theorem 1.2 per-Λ form implementation (`Hyp_OF ≅ Cospan-Alg` per Λ).

## Pre-workspace history

Tags v0.3.0 through v0.9.0 (2026-04-01 through 2026-04-07) predate the workspace restructuring. See `git tag --sort=-creatordate` and individual commit messages for scope of those releases.

[Unreleased]: https://github.com/tsondru/catgraph/compare/v0.12.0...HEAD
[0.12.0]: https://github.com/tsondru/catgraph/compare/v0.11.4...v0.12.0
[0.11.4]: https://github.com/tsondru/catgraph/releases/tag/v0.11.4
[0.11.3]: https://github.com/tsondru/catgraph/releases/tag/v0.11.3
[0.11.2]: https://github.com/tsondru/catgraph/releases/tag/v0.11.2
[0.11.1]: https://github.com/tsondru/catgraph/releases/tag/v0.11.1
[0.11.0]: https://github.com/tsondru/catgraph/releases/tag/v0.11.0
[0.10.6]: https://github.com/tsondru/catgraph/releases/tag/v0.10.6
[0.10.5]: https://github.com/tsondru/catgraph/releases/tag/v0.10.5
[0.10.4]: https://github.com/tsondru/catgraph/releases/tag/v0.10.4
[0.10.3]: https://github.com/tsondru/catgraph/releases/tag/v0.10.3
[0.10.0]: https://github.com/tsondru/catgraph/releases/tag/v0.10.0
