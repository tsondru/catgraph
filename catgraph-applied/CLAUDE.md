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

## Alignment with F&S applied CT paper

A specific applied-CT paper (F&S or equivalent) will be added to `docs/` to anchor the design. When that paper lands, draft `docs/FONG-SPIVAK-APPLIED-AUDIT.md` following the same template as `catgraph/docs/FONG-SPIVAK-AUDIT.md`. Cross-link from the core audit's "Reconciliation with catgraph-applied" section.

Until then, these modules are implementation-first and will be restructured once the paper anchor is chosen.

## Build

```sh
cargo test -p catgraph-applied
cargo clippy -p catgraph-applied -- -W clippy::pedantic
cargo test -p catgraph-applied --examples
cargo bench -p catgraph-applied --no-run
```
