# catgraph-physics

Wolfram-physics extensions for [catgraph](../catgraph/): hypergraph DPO rewriting, multiway evolution tracking, gauge theory, and branchial spectral analysis.

Part of the [catgraph workspace](https://github.com/tsondru/catgraph).

## Modules

| Module | Purpose |
|--------|---------|
| `hypergraph/` | Hypergraph DPO rewriting, evolution tracking, categorical span/cospan bridges, lattice gauge theory |
| `multiway/` | Generic multiway (non-deterministic) evolution graphs, branchial foliation, Ollivier-Ricci curvature, Wasserstein transport |
| `multiway/branchial_spectrum.rs` | Graph Laplacian eigendecomposition: algebraic connectivity (λ₂), spectral gap, Fiedler vector, spectral clustering |
| `multiway/branchial_analysis.rs` | Graph algorithms via rustworkx-core: greedy coloring, k-core decomposition, articulation points |

## Dependencies

- `catgraph` — core F&S types (`Composable`, `Cospan`, `Span`)
- `nalgebra` + `nalgebra-sparse` — spectral analysis
- `petgraph` + `rustworkx-core` — graph algorithms

## Build

```sh
cargo test -p catgraph-physics
cargo clippy -p catgraph-physics -- -W clippy::pedantic
cargo bench -p catgraph-physics --bench wasserstein_bench
```

## License

MIT — see [LICENSE](../LICENSE).
