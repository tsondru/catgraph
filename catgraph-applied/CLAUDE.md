# catgraph-applied

Applied category theory extensions for catgraph. Workspace member of [catgraph](../CLAUDE.md).

## Scope

Modules that build on catgraph's F&S 2019 core (cospans, spans, Frobenius, hypergraph categories) but are **not** part of the 2019 paper's numbered content. This crate is the applied-CT complement to the strict F&S core.

**In scope (Tier 1, v0.3.x):**
- `decorated_cospan.rs` — generic `Decoration` trait + `DecoratedCospan<Lambda, D>` realizing F&S Def 6.75 + Thm 6.77 (v0.3.0); `D::pushforward` wired through `compose` via `Cospan::compose_with_quotient` (v0.3.1)
- `wiring_diagram.rs` — operadic substitution on named cospans
- `petri_net.rs` — place/transition nets with cospan bridge, firing, reachability, parallel/sequential composition. `HypergraphCategory` impl + `PetriDecoration` bridge to `DecoratedCospan` (v0.3.0); direct `permute_side` and `Transition::relabel` arc dedup (v0.3.1)
- `temperley_lieb.rs` — Temperley-Lieb / Brauer algebra via perfect matchings
- `linear_combination.rs` — formal linear combinations over a coefficient ring (used internally by `temperley_lieb`)

**In scope (Tier 2, v0.4.0):**
- `e1_operad.rs` — little-intervals operad (E₁) with public `arity()`, `sub_intervals()`, `Clone`
- `e2_operad.rs` — little-disks operad (E₂) with public `arity_of()`, `sub_circles()`, `Clone`
- `prop.rs` — props + free prop on a signature (F&S Def 5.2, Def 5.25). `PropSignature` trait, arity-tracked `PropExpr<G>`, `Free<G>` smart constructors, full catgraph trait-hierarchy impl. `PropSignature` supertrait widened to `Eq + Hash` in v0.5.1.
- `operad_algebra.rs` — single-sorted `OperadAlgebra<O, Input>` trait (F&S Def 6.99) with concrete `CircAlgebra` implementing Ex 6.100 for `WiringDiagram` via outer-port counts.
- `operad_functor.rs` — `OperadFunctor<O1, O2, Input>` trait (F&S Rough Def 6.98) with concrete `E1ToE2` canonical inclusion. `start_name` offset lets the two branches of `F(o ∘_i q) = F(o) ∘_i F(q)` share a substitution without colliding on E₂'s unique-name invariant; literal geometric functoriality is verified by comparing canonical disk positions modulo naming.

**In scope (Tier 3, v0.5.x):**
- `rig.rs` — `Rig` trait (F&S Def 5.36) as a blanket impl over `num_traits::{Zero, One}` + `Add` + `Mul`. Four concrete instances: `BoolRig` (∨,∧), `UnitInterval` ([0,1] Viterbi), `Tropical` ([0,∞], min, +), `F64Rig`. The three f64-wrapping rigs provide manual `Eq + Hash` via bit-exact `to_bits()` (v0.5.0/v0.5.1).
- `sfg.rs` — `SignalFlowGraph<R>` (F&S Def 5.45) — free prop on the 5 primitive signal-flow generators from Eq 5.52 (v0.5.0).
- `mat.rs` — `MatR<R>` (F&S Def 5.50) — pure-rig matrix prop over any `Rig` R (v0.5.0).
- `sfg_to_mat.rs` — `sfg_to_mat` functor `S: SFG_R → Mat(R)` (F&S Thm 5.53; v0.5.0).
- `graphical_linalg.rs` — `matr_presentation<R>` — 16-equation F&S Thm 5.60 presentation of Mat(R) (v0.5.0). Closed semantically in v0.5.2 via the Functorial engine.
- `mat_f64.rs` (feature `f64-rig`) — nalgebra bridge for `MatR<F64Rig>` (v0.5.0).
- `prop/presentation/mod.rs` — `Presentation<G>` with 9-rule SMC canonical form + user equations + `NormalizeEngine` selector (Structural / CongruenceClosure; v0.5.0/v0.5.1). Rule 9 (`Identity(m) ⊗ Identity(n) → Identity(m+n)`) added v0.5.1.
- `prop/presentation/kb.rs` — congruence-closure decision procedure (DST 1980 signature-table variant) as default `eq_mod` backend (v0.5.1). v0.5.2 adds atom-canonical `smc_refine` fixpoint (~44% BoolRig d=2 collision reduction).
- `prop/presentation/smc_nf.rs` — Layer 1 Joyal-Street string-diagram NF — canonicalizes `PropExpr` up to SMC coherence (associator, unitors, interchange, braid naturality, σ²=id). Used as `Presentation::eq_mod` short-circuit (v0.5.2).
- `prop/presentation/functorial.rs` — `CompleteFunctor<G>` trait + `MatrixNFFunctor<R>` concrete instance wrapping `sfg_to_mat`. `Presentation::eq_mod_functorial<F>` method dispatches through any complete functor — complete-by-theorem decision procedure for `Free(Σ_SFG)/⟨E_{17}⟩ ≅ Mat(R)` (Baez-Erbele 2015; v0.5.2).
- `enriched.rs` — `EnrichedCategory<V: Rig>` trait + `HomMap<O, V>` finite realization (F&S §1.1, §2.4; v0.5.1). Phase 6 catgraph-magnitude substrate.
- `lawvere_metric.rs` — `LawvereMetricSpace<T>` over `Tropical` — triangle-inequality verifier + `-ln π` embedding from `UnitInterval` (Lawvere 1973; v0.5.1).

**Out of scope:**
- F&S core types (cospans, spans, Frobenius, hypergraph categories, compact closed, equivalence) → `catgraph`
- Wolfram-physics extensions (hypergraph rewriting, multiway, gauge, branchial) → `catgraph-physics`
- Persistence → [catgraph-surreal](https://github.com/tsondru/catgraph-surreal)

## Paper alignment

Anchored to Fong & Spivak, *Seven Sketches in Compositionality* (arXiv:1803.05316v3, 2018) — Chapters 4–6. See [`docs/SEVEN-SKETCHES-AUDIT.md`](docs/SEVEN-SKETCHES-AUDIT.md) for the section-by-section audit. Cross-linked from [`catgraph/docs/FONG-SPIVAK-AUDIT.md`](../catgraph/docs/FONG-SPIVAK-AUDIT.md) "Reconciliation" section.

Key alignments:
- `decorated_cospan` → §6.4 Def 6.75 + Thm 6.77 (generic `DecoratedCospan<Lambda, D>` hypergraph category)
- `wiring_diagram` → §6.5 Ex 6.94 (Cospan operad), §6.3.2, §4.4.2
- `petri_net::HypergraphCategory` → §6.3 Def 6.60 via Thm 6.77 (PetriNet as hypergraph category via `PetriDecoration`)
- `petri_net` → §6.4 Def 6.75 (decorated cospans, specialized); further reading [BFP16, BP17]
- `temperley_lieb` → §6.3 (spider-adjacent); Jones/Kauffman/Brauer literature
- `e1_operad` / `e2_operad` → §6.5 Rough Def 6.91; May/Boardman-Vogt literature
- `linear_combination` → §5.3.1 (rig infrastructure)
- `prop` → §5.2 Def 5.2 (prop) + Def 5.25 (`Free(G)`); v0.4.0
- `prop::presentation` → §5.2 Def 5.33 (presentation); v0.5.0 + CC backend v0.5.1 + Layer 1 NF v0.5.2
- `operad_algebra` → §6.5 Def 6.99 (operad algebra) + Ex 6.100 (Circ); v0.4.0
- `operad_functor` → §6.5 Rough Def 6.98 (operad functor); v0.4.0
- `rig` → §5.3.1 Def 5.36 (rig); v0.5.0
- `sfg` → §5.3 Def 5.45 (signal flow graphs); v0.5.0
- `mat` + `sfg_to_mat` → §5.3 Def 5.50 + Thm 5.53 (matrix prop + functor); v0.5.0
- `graphical_linalg` → §5.4 Thm 5.60 (16-equation Mat(R) presentation); v0.5.0 + Functorial engine closes the theorem v0.5.2
- `prop::presentation::functorial::MatrixNFFunctor` → §5.4 Thm 5.60 decision procedure (Baez-Erbele 2015); v0.5.2
- `enriched` → §1.1, §2.4, Rough Def 4.51 (V-enriched categories); v0.5.1
- `lawvere_metric` → §1.3–1.4 pedagogical anchor (Lawvere metric spaces); v0.5.1

## Build

```sh
cargo test -p catgraph-applied
cargo clippy -p catgraph-applied -- -W clippy::pedantic
cargo test -p catgraph-applied --examples
cargo bench -p catgraph-applied --no-run
```
