# catgraph workspace

This workspace contains the **catgraph** crate, a Rust implementation of [Fong & Spivak, *Hypergraph Categories* (2019)](https://arxiv.org/abs/1806.08304).

## Members

| Crate | Path | Purpose |
|---|---|---|
| [`catgraph`](catgraph/) | `catgraph/` | Strict Fong-Spivak 2019 paper implementation: cospans, spans, Frobenius algebras, hypergraph categories, Theorem 1.2 equivalence |
| [`catgraph-physics`](catgraph-physics/) | `catgraph-physics/` | Wolfram-physics extensions: hypergraph DPO rewriting, multiway evolution, gauge theory, branchial spectral analysis |

## Sibling repositories

These are separate repos that depend on `catgraph` and/or `catgraph-physics`:

| Repo | Purpose |
|---|---|
| [catgraph-surreal](https://github.com/tsondru/catgraph-surreal) | SurrealDB persistence layer for catgraph and catgraph-physics types |
| [irreducible](https://github.com/tsondru/irreducible) | Computational irreducibility framework (Gorard 2023) using catgraph and catgraph-physics |

## Build

```sh
cargo build --workspace
cargo test --workspace
```

See [`catgraph/README.md`](catgraph/README.md) for the catgraph crate's own documentation.

## License

MIT — see [LICENSE](LICENSE).
