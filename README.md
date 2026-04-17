# catgraph workspace

Category-theoretic graph structures in Rust. The **catgraph** crate (v0.11.2, slim baseline) is a strict Rust implementation of [Fong & Spivak, *Hypergraph Categories* (2019)](https://arxiv.org/abs/1806.08304). Applied-CT extensions (catgraph-applied v0.3.0) track [Fong & Spivak, *Seven Sketches in Compositionality* (2018)](https://arxiv.org/abs/1803.05316). Wolfram-physics extensions live in a third sibling workspace crate.

## Members

| Crate | Path | Purpose |
|---|---|---|
| [`catgraph`](catgraph/) v0.11.2 | `catgraph/` | Strict Fong-Spivak 2019 paper implementation: cospans, spans, Frobenius algebras, hypergraph categories, Theorem 1.2 equivalence, spider theorem (Thm 6.55) |
| [`catgraph-physics`](catgraph-physics/) v0.2.1 | `catgraph-physics/` | Wolfram-physics extensions: hypergraph DPO rewriting, multiway evolution, gauge theory, branchial spectral analysis |
| [`catgraph-applied`](catgraph-applied/) v0.3.0 | `catgraph-applied/` | Applied CT extensions: generic `DecoratedCospan<F>` (Def 6.75, Thm 6.77), Petri nets as hypergraph category, wiring diagrams, E_n operads, Temperley-Lieb, linear combinations |

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
