# catgraph-applied

Applied category theory extensions for [catgraph](../catgraph). Anchored to [Fong & Spivak, *Seven Sketches in Compositionality* (arXiv:1803.05316v3, 2018)](https://arxiv.org/abs/1803.05316), Chapters 4–6.

## Overview

This crate packages applied-CT modules that build on catgraph's strict Fong-Spivak 2019 core but are not part of the 2019 paper's numbered content. It is the applied-CT complement to the F&S core crate.

## Modules

| Module | Purpose |
|---|---|
| `decorated_cospan` | Generic `Decoration` trait + `DecoratedCospan<Lambda, D>` realizing F&S Def 6.75 + Thm 6.77 |
| `wiring_diagram` | Operadic substitution built on named cospans |
| `petri_net` | Place/transition nets with cospan bridge, firing, reachability, parallel/sequential composition, `HypergraphCategory` impl, `PetriDecoration` bridge to `DecoratedCospan` |
| `temperley_lieb` | Temperley-Lieb / Brauer algebra via perfect matchings |
| `linear_combination` | Formal linear combinations over a coefficient ring |
| `e1_operad` | Little-intervals operad (E₁) |
| `e2_operad` | Little-disks operad (E₂) |
| `prop` | Symmetric strict monoidal categories with `Ob = ℕ` and the free prop `Free(G)` on a signature (F&S Def 5.2, Def 5.25; v0.4.0); `Presentation<G>` with 9-rule SMC quotient (Def 5.33; v0.5.0, Rule 9 added v0.5.1) |
| `operad_algebra` | Single-sorted operad algebras `F : O → Set` with concrete `CircAlgebra` for `WiringDiagram` (F&S Def 6.99, Ex 6.100; v0.4.0) |
| `operad_functor` | Functors between operads with the canonical `E₁ ↪ E₂` inclusion (F&S Rough Def 6.98; v0.4.0) |
| `rig` | `Rig` trait (semiring) + `BoolRig`, `UnitInterval`, `Tropical`, `F64Rig` instances (F&S Def 5.36; v0.5.0) |
| `sfg` | `SignalFlowGraph<R>` — free prop on signal-flow generators (F&S Def 5.45; v0.5.0) |
| `mat` | `MatR<R>` — pure-rig matrix prop over any `Rig` R (F&S Def 5.50; v0.5.0) |
| `sfg_to_mat` | `sfg_to_mat` functor `S: SFG_R → Mat(R)` (F&S Thm 5.53; v0.5.0) |
| `graphical_linalg` | `matr_presentation<R>` — 16-equation Thm 5.60 presentation of Mat(R) (F&S §5.4; v0.5.0; closed v0.5.2 via the Functorial engine — see `prop::presentation::functorial`) |
| `mat_f64` (feature `f64-rig`) | nalgebra bridge for `MatR<F64Rig>`: determinant, inverse, `DMatrix` roundtrip (v0.5.0) |
| `prop::presentation::kb` | Congruence-closure decision procedure (DST 1980 signature-table variant) — default `eq_mod` backend since v0.5.1; v0.5.2 adds an atom-canonical `smc_refine` fixpoint (~44% BoolRig d=2 collision reduction) |
| `prop::presentation::smc_nf` | Layer 1 Joyal-Street string-diagram normal form — canonicalizes `PropExpr` up to SMC coherence (associator, unitors, interchange, braid naturality, σ²=id) (JS 1991 Part I, Selinger 2011; v0.5.2) |
| `prop::presentation::functorial` | `CompleteFunctor<G>` trait + `MatrixNFFunctor<R>` — opt-in semantic decision engine for prop-equality via `Presentation::eq_mod_functorial`. Complete by theorem for Mat(R) (Baez-Erbele 2015 / F&S Thm 5.60; v0.5.2) |
| `enriched` | `EnrichedCategory<V>` trait + `HomMap<O, V>` finite realization (F&S §1.1, §2.4; v0.5.1) |
| `lawvere_metric` | `LawvereMetricSpace<T>` over `Tropical` — triangle-inequality verifier + `-ln π` embedding from `UnitInterval` (Lawvere 1973; v0.5.1) |

### New in v0.5.3

- `F64Rig`: `Neg`, `Sub`, `Div`, and `From<f64>` impls exposing the ring and
  field operations that catgraph-magnitude v0.1.0 requires for `mobius_function`
  Gaussian elimination. The math-level ring property was already present (the
  `verify_axioms_f64_rig_sample` test exercises `F64Rig(-1.0)`); this version
  exposes the ring + field operations to Rust's type system. The ring/field bound
  stays off `Rig` itself — only `F64Rig` carries it.

### New in v0.5.2

Three independent tracks, all additive (no API break from v0.5.1):

- **Layer 1 Joyal-Street string-diagram NF** (`prop::presentation::smc_nf`) —
  total function `PropExpr → StringDiagram` canonicalizing up to SMC
  coherence. 18 paper-cited regression tests + 6 proptest coverage tests.
- **Option A atom-canonical CC refinement** in `kb::CongruenceClosure` —
  `propagate_fixpoint` outer loop alternating congruence propagation with a
  post-merge `smc_refine` pass. Measured ~44% BoolRig d=2 collision
  reduction. `Presentation::eq_mod` CC branch also gains a Layer-1-NF
  short-circuit.
- **Functorial engine** (`prop::presentation::functorial`) —
  `CompleteFunctor<G>` trait + `MatrixNFFunctor<R>` concrete wrapping
  `sfg_to_mat` as a complete-by-theorem decision procedure for Mat(R).
  Opt-in via `Presentation::eq_mod_functorial<F>` — complements the
  syntactic `eq_mod`.

**§5.4 Thm 5.60 closed** via the Functorial engine: the seven-sketches
audit now tracks **87% implementable DONE / 7% PARTIAL / 7% MISSING**.

The 12 previously-`thm_5_60_faithful_*` integration tests were renamed to
`cc_completeness_tracking_*` in v0.5.2 to reflect that they measure the
default CC engine's syntactic incompleteness vs the matrix ground truth —
they are NOT Thm 5.60 verification (Baez-Erbele proved that abstractly) and
stay `#[ignore]`'d as diagnostic.

### New in v0.5.1

- `prop::presentation::kb` — congruence-closure decision procedure for
  `Presentation` (replaces bounded structural rewriting as the default
  `eq_mod` backend).
- `enriched::EnrichedCategory<V>` — V-enriched categories over a `Rig`.
  Object-safe for heterogeneous `dyn` collections.
- `lawvere_metric::LawvereMetricSpace<T>` — Lawvere metric spaces over
  `Tropical` with triangle-inequality verification.

**BREAKING:** `Presentation::normalize` / `eq_mod` signatures changed in
v0.5.1. `PropSignature` widened to `Eq + Hash`. See `CHANGELOG.md` for
migration.

## Dependency on catgraph

Every module depends on catgraph's public API:

- `Cospan`, `NamedCospan`, `Span`, `Rel` — pushout/pullback composition
- `Frobenius` generators — operadic composition of SMCs (Prop 3.8)
- `HypergraphCategory` trait — target for semantic functors
- `Operadic` trait — abstract substitution interface (concrete impls live here)
- `compact_closed` cup/cap — string-diagram rewriting (TL, wiring)

## Paper alignment

See [`docs/SEVEN-SKETCHES-AUDIT.md`](docs/SEVEN-SKETCHES-AUDIT.md) for the section-by-section Seven Sketches coverage audit (Chapters 4–6, 60 items tracked; v0.5.2 headline: 87% of implementable items DONE, §5.4 Thm 5.60 closed via Functorial engine). Cross-linked from [`../catgraph/docs/FONG-SPIVAK-AUDIT.md`](../catgraph/docs/FONG-SPIVAK-AUDIT.md) "Reconciliation" section.

## Changelog

See [`CHANGELOG.md`](CHANGELOG.md) for release history.

## Build

```sh
cargo test -p catgraph-applied
cargo clippy -p catgraph-applied -- -W clippy::pedantic
```

## WASM support (v0.3.3+)

`[features] parallel` (default-on) gates the `rayon` + `rayon-cond`
dependencies and the four `CondIterator` call sites in
`linear_combination::Mul::mul`, `linear_combination::linear_combine`, and
`temperley_lieb::BrauerMorphism::non_crossing` (source + target sides).
Disable with `--no-default-features` for single-threaded WASI hosts.

```sh
cargo build --lib -p catgraph-applied --target wasm32-wasip1-threads
cargo build --lib -p catgraph-applied --target wasm32-wasip1 --no-default-features
```

See `examples/wasi_smoke_applied.rs` for a minimal `LinearCombination`
multiplication smoke test exercising the `CondIterator` parallel arm.

## License

MIT.
