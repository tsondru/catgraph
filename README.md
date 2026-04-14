# catgraph workspace

Category-theoretic graph structures in Rust. The **catgraph** crate (v0.11.0 slim baseline) is a strict Rust implementation of [Fong & Spivak, *Hypergraph Categories* (2019)](https://arxiv.org/abs/1806.08304). Applied-CT extensions and Wolfram-physics extensions live in sibling workspace crates.

## Members

| Crate | Path | Purpose |
|---|---|---|
| [`catgraph`](catgraph/) | `catgraph/` | Strict Fong-Spivak 2019 paper implementation: cospans, spans, Frobenius algebras, hypergraph categories, Theorem 1.2 equivalence |
| [`catgraph-physics`](catgraph-physics/) | `catgraph-physics/` | Wolfram-physics extensions: hypergraph DPO rewriting, multiway evolution, gauge theory, branchial spectral analysis |
| [`catgraph-applied`](catgraph-applied/) | `catgraph-applied/` | Applied CT extensions: Petri nets, wiring diagrams, E_n operads, Temperley-Lieb, linear combinations |

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
cargo clippy --workspace -- -W clippy::pedantic
```

Each crate has its own README and CLAUDE.md:

- [`catgraph/README.md`](catgraph/README.md) — strict F&S 2019 implementation
- [`catgraph-physics/README.md`](catgraph-physics/README.md) — Wolfram-physics extensions
- [`catgraph-applied/README.md`](catgraph-applied/README.md) — applied-CT extensions

## License

MIT — see [LICENSE](LICENSE).
