# catgraph-applied

Applied category theory extensions for catgraph. Workspace member of [catgraph](../CLAUDE.md).

## Scope

Modules that build on catgraph's F&S 2019 core (cospans, spans, Frobenius, hypergraph categories) but are **not** part of the 2019 paper's numbered content. This crate is the applied-CT complement to the strict F&S core.

**In scope:**
- `wiring_diagram.rs` — operadic substitution on named cospans
- `petri_net.rs` — place/transition nets with cospan bridge, firing, reachability, parallel/sequential composition
- `temperley_lieb.rs` — Temperley-Lieb / Brauer algebra via perfect matchings
- `linear_combination.rs` — formal linear combinations over a coefficient ring (used internally by `temperley_lieb`)
- `e1_operad.rs` — little-intervals operad (E₁)
- `e2_operad.rs` — little-disks operad (E₂)

**Out of scope:**
- F&S core types (cospans, spans, Frobenius, hypergraph categories, compact closed, equivalence) → `catgraph`
- Wolfram-physics extensions (hypergraph rewriting, multiway, gauge, branchial) → `catgraph-physics`
- Persistence → [catgraph-surreal](https://github.com/tsondru/catgraph-surreal)

## Paper alignment

Anchored to Fong & Spivak, *Seven Sketches in Compositionality* (arXiv:1803.05316v3, 2018) — Chapters 4–6. See [`docs/SEVEN-SKETCHES-AUDIT.md`](docs/SEVEN-SKETCHES-AUDIT.md) for the section-by-section audit. Cross-linked from [`catgraph/docs/FONG-SPIVAK-AUDIT.md`](../catgraph/docs/FONG-SPIVAK-AUDIT.md) "Reconciliation" section.

Key alignments:
- `wiring_diagram` → §6.5 Ex 6.94 (Cospan operad), §6.3.2, §4.4.2
- `petri_net` → §6.4 Def 6.75 (decorated cospans, specialized); further reading [BFP16, BP17]
- `temperley_lieb` → §6.3 (spider-adjacent); Jones/Kauffman/Brauer literature
- `e1_operad` / `e2_operad` → §6.5 Rough Def 6.91; May/Boardman-Vogt literature
- `linear_combination` → §5.3.1 (rig infrastructure)

## Build

```sh
cargo test -p catgraph-applied
cargo clippy -p catgraph-applied -- -W clippy::pedantic
cargo test -p catgraph-applied --examples
cargo bench -p catgraph-applied --no-run
```
