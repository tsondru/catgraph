# catgraph-applied

Applied category theory extensions for catgraph. Workspace member of [catgraph](../CLAUDE.md).

## Scope

Modules that build on catgraph's F&S 2019 core (cospans, spans, Frobenius, hypergraph categories) but are **not** part of the 2019 paper's numbered content. This crate is the applied-CT complement to the strict F&S core.

**In scope:**
- `decorated_cospan.rs` — generic `Decoration` trait + `DecoratedCospan<Lambda, D>` realizing F&S Def 6.75 + Thm 6.77 (v0.3.0); `D::pushforward` wired through `compose` via `Cospan::compose_with_quotient` (v0.3.1)
- `wiring_diagram.rs` — operadic substitution on named cospans
- `petri_net.rs` — place/transition nets with cospan bridge, firing, reachability, parallel/sequential composition. `HypergraphCategory` impl + `PetriDecoration` bridge to `DecoratedCospan` (v0.3.0); direct `permute_side` and `Transition::relabel` arc dedup (v0.3.1)
- `temperley_lieb.rs` — Temperley-Lieb / Brauer algebra via perfect matchings
- `linear_combination.rs` — formal linear combinations over a coefficient ring (used internally by `temperley_lieb`)
- `e1_operad.rs` — little-intervals operad (E₁) with public `arity()`, `sub_intervals()`, `Clone` (v0.4.0)
- `e2_operad.rs` — little-disks operad (E₂) with public `arity_of()`, `sub_circles()`, `Clone` (v0.4.0)
- `prop.rs` — props + free prop on a signature (F&S Def 5.2, Def 5.25; v0.4.0). `PropSignature` trait, arity-tracked `PropExpr<G>`, `Free<G>` smart constructors, full catgraph trait-hierarchy impl. Equality is structural — the SMC quotient (interchange + unitors + braiding naturality) is v0.5.0 work.
- `operad_algebra.rs` — single-sorted `OperadAlgebra<O, Input>` trait (F&S Def 6.99; v0.4.0) with concrete `CircAlgebra` implementing Ex 6.100 for `WiringDiagram` via outer-port counts.
- `operad_functor.rs` — `OperadFunctor<O1, O2, Input>` trait (F&S Rough Def 6.98; v0.4.0) with concrete `E1ToE2` canonical inclusion. `start_name` offset lets the two branches of `F(o ∘_i q) = F(o) ∘_i F(q)` share a substitution without colliding on E₂'s unique-name invariant; literal geometric functoriality is verified by comparing canonical disk positions modulo naming.

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
- `operad_algebra` → §6.5 Def 6.99 (operad algebra) + Ex 6.100 (Circ); v0.4.0
- `operad_functor` → §6.5 Rough Def 6.98 (operad functor); v0.4.0

## Build

```sh
cargo test -p catgraph-applied
cargo clippy -p catgraph-applied -- -W clippy::pedantic
cargo test -p catgraph-applied --examples
cargo bench -p catgraph-applied --no-run
```
