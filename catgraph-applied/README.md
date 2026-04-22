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
| `prop` | Symmetric strict monoidal categories with `Ob = ℕ` and the free prop `Free(G)` on a signature (F&S Def 5.2, Def 5.25; v0.4.0); `Presentation<G>` with 8-rule SMC quotient (Def 5.33; v0.5.0) |
| `operad_algebra` | Single-sorted operad algebras `F : O → Set` with concrete `CircAlgebra` for `WiringDiagram` (F&S Def 6.99, Ex 6.100; v0.4.0) |
| `operad_functor` | Functors between operads with the canonical `E₁ ↪ E₂` inclusion (F&S Rough Def 6.98; v0.4.0) |
| `rig` | `Rig` trait (semiring) + `BoolRig`, `UnitInterval`, `Tropical`, `F64Rig` instances (F&S Def 5.36; v0.5.0) |
| `sfg` | `SignalFlowGraph<R>` — free prop on signal-flow generators (F&S Def 5.45; v0.5.0) |
| `mat` | `MatR<R>` — pure-rig matrix prop over any `Rig` R (F&S Def 5.50; v0.5.0) |
| `sfg_to_mat` | `sfg_to_mat` functor `S: SFG_R → Mat(R)` (F&S Thm 5.53; v0.5.0) |
| `graphical_linalg` | `matr_presentation<R>` — 16-equation Thm 5.60 presentation of Mat(R) (F&S §5.4; v0.5.0, PARTIAL — string-diagram normal form deferred to v0.5.2) |
| `mat_f64` (feature `f64-rig`) | nalgebra bridge for `MatR<F64Rig>`: determinant, inverse, `DMatrix` roundtrip (v0.5.0) |
| `prop::presentation::kb` | Congruence-closure decision procedure (DST 1980 signature-table variant) — default `eq_mod` backend since v0.5.1 |
| `enriched` | `EnrichedCategory<V>` trait + `HomMap<O, V>` finite realization (F&S §1.1, §2.4; v0.5.1) |
| `lawvere_metric` | `LawvereMetricSpace<T>` over `Tropical` — triangle-inequality verifier + `-ln π` embedding from `UnitInterval` (Lawvere 1973; v0.5.1) |

### New in v0.5.1

- `prop::presentation::kb` — congruence-closure decision procedure for
  `Presentation` (replaces bounded structural rewriting as the default
  `eq_mod` backend).
- `enriched::EnrichedCategory<V>` — V-enriched categories over a `Rig`.
  Object-safe for heterogeneous `dyn` collections.
- `lawvere_metric::LawvereMetricSpace<T>` — Lawvere metric spaces over
  `Tropical` with triangle-inequality verification.

**BREAKING:** `Presentation::normalize` / `eq_mod` signatures changed.
`PropSignature` widened to `Eq + Hash`. See `CHANGELOG.md` for migration.

**Known gap:** Thm 5.60 faithfulness tests remain `#[ignore]`'d pending
SMC string-diagram normal form (v0.5.2).

## Dependency on catgraph

Every module depends on catgraph's public API:

- `Cospan`, `NamedCospan`, `Span`, `Rel` — pushout/pullback composition
- `Frobenius` generators — operadic composition of SMCs (Prop 3.8)
- `HypergraphCategory` trait — target for semantic functors
- `Operadic` trait — abstract substitution interface (concrete impls live here)
- `compact_closed` cup/cap — string-diagram rewriting (TL, wiring)

## Paper alignment

See [`docs/SEVEN-SKETCHES-AUDIT.md`](docs/SEVEN-SKETCHES-AUDIT.md) for the section-by-section Seven Sketches coverage audit (Chapters 4–6, 57 items tracked; v0.5.0 headline: 81% of implementable items DONE). Cross-linked from [`../catgraph/docs/FONG-SPIVAK-AUDIT.md`](../catgraph/docs/FONG-SPIVAK-AUDIT.md) "Reconciliation" section.

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
