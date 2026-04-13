# catgraph-physics

Wolfram-physics extensions for catgraph. Workspace member of [catgraph](../CLAUDE.md).

## Scope

Hypergraph DPO rewriting, multiway evolution, gauge theory, branchial spectral analysis. Depends on catgraph core for `Composable`, `Cospan`, `Span`.

**In scope:**
- `hypergraph/` — `Hypergraph`, `RewriteRule`, `HypergraphEvolution`, `HypergraphLattice` (gauge), categorical bridges (`rewrite_span.rs`, `evolution_cospan.rs`, `multiway_cospan.rs`)
- `multiway/` — `MultiwayEvolutionGraph`, `BranchialGraph`, `OllivierRicciCurvature`, `wasserstein_1`, `BranchialSpectrum`, graph algorithms (coloring, k-core, articulation points)

**Out of scope:**
- F&S core types (cospans, spans, Frobenius, etc.) → `catgraph`
- Petri nets, operads, Temperley-Lieb, wiring diagrams → `catgraph-applied` (future)
- Persistence → [catgraph-surreal](https://github.com/tsondru/catgraph-surreal)
- Computational irreducibility → [irreducible](https://github.com/tsondru/irreducible)

## Build

```sh
cargo test -p catgraph-physics
cargo clippy -p catgraph-physics -- -W clippy::pedantic
```
