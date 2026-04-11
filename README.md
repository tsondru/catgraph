# catgraph workspace

This workspace contains the **catgraph** crate, a Rust implementation of [Fong & Spivak, *Hypergraph Categories* (2019)](https://arxiv.org/abs/1806.08304).

## Members

| Crate | Path | Purpose |
|---|---|---|
| [`catgraph`](catgraph/) | `catgraph/` | Strict Fong-Spivak 2019 paper implementation: cospans, spans, Frobenius algebras, hypergraph categories, Theorem 1.2 equivalence |

## Sibling repositories

These are separate repos that depend on `catgraph`:

| Repo | Purpose |
|---|---|
| [catgraph-surreal](https://github.com/tsondru/catgraph-surreal) | SurrealDB persistence layer for catgraph types |
| [irreducible](https://github.com/tsondru/irreducible) | Computational irreducibility framework (Gorard 2023) using catgraph |

## Build

```sh
cargo build --workspace
cargo test --workspace
```

See [`catgraph/README.md`](catgraph/README.md) for the catgraph crate's own documentation.

## License

MIT — see [LICENSE](LICENSE).
