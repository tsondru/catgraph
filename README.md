# catgraph workspace

Category-theoretic graph structures in Rust. The **catgraph** crate (v0.12.0, slim baseline) is a strict Rust implementation of [Fong & Spivak, *Hypergraph Categories* (2019)](https://arxiv.org/abs/1806.08304). Applied-CT extensions (catgraph-applied v0.5.3) track [Fong & Spivak, *Seven Sketches in Compositionality* (2018)](https://arxiv.org/abs/1803.05316). Wolfram-physics extensions live in a third workspace crate. Magnitude of enriched categories (catgraph-magnitude v0.1.0) is anchored to [Bradley & Vigneaux, *Magnitude of Language Models* (2025)](https://arxiv.org/abs/2501.06662).

## Members

| Crate | Path | Purpose |
|---|---|---|
| [`catgraph`](catgraph/) v0.12.0 | `catgraph/` | Strict Fong-Spivak 2019 paper implementation: cospans, spans, Frobenius algebras, hypergraph categories, Theorem 1.2 equivalence, spider theorem (Thm 6.55), `Corel<Λ>` (F&S 2018 Ex 6.64; v0.12.0), `Cospan::compose_with_quotient` additive API, `parallel` feature (WASM + native, W.1) |
| [`catgraph-physics`](catgraph-physics/) v0.2.2 | `catgraph-physics/` | Wolfram-physics extensions: hypergraph DPO rewriting, multiway evolution, gauge theory, branchial spectral analysis |
| [`catgraph-applied`](catgraph-applied/) v0.5.3 | `catgraph-applied/` | Applied CT extensions. **Tier 1** (v0.3.x): generic `DecoratedCospan<F>` (Def 6.75, Thm 6.77), Petri nets as hypergraph category, wiring diagrams, E_n operads, Temperley-Lieb, linear combinations. **Tier 2** (v0.4.0): props + `Free(G)` (Def 5.2, 5.25), `OperadAlgebra` with Circ (Def 6.99, Ex 6.100), `OperadFunctor` with canonical E₁↪E₂ (Rough Def 6.98). **Tier 3** (v0.5.x): `Rig` + 4 concrete rigs (Def 5.36), `SignalFlowGraph<R>` (Def 5.45), `MatR<R>` (Def 5.50), `sfg_to_mat` functor (Thm 5.53), `Presentation<G>` with CC decision engine + Layer-1 Joyal-Street NF short-circuit, `EnrichedCategory<V>` + `LawvereMetricSpace<T>`, `CompleteFunctor<G>` + `MatrixNFFunctor<R>` closing Thm 5.60 semantically (v0.5.2); `F64Rig` ring + field ops prereq for catgraph-magnitude (v0.5.3) |
| [`catgraph-magnitude`](catgraph-magnitude/) v0.1.0 | `catgraph-magnitude/` | Magnitude of enriched categories. Anchored to BV 2025: `Mag(tM)` via Tsallis q-entropy decomposition (Prop 3.10), Shannon recovery (Rem 3.11), Möbius inversion. `LmCategory` materialized transition table. `WeightedCospan<Λ, Q>` for agent-coalition weight graphs. No tokio, no serde, no rayon. |

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
- [`catgraph-magnitude/README.md`](catgraph-magnitude/README.md) — BV 2025 magnitude

## License

MIT — see [LICENSE](LICENSE).
