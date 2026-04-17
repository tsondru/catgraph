# catgraph workspace

Category-theoretic graph structures in Rust. The core [`catgraph`](catgraph/) crate (v0.11.2) is a strict implementation of [Fong & Spivak, *Hypergraph Categories* (2019)](https://arxiv.org/abs/1806.08304). [`catgraph-applied`](catgraph-applied/) (v0.3.0) anchors applied-CT extensions to [Fong & Spivak, *Seven Sketches in Compositionality* (2018)](https://arxiv.org/abs/1803.05316). Wolfram-physics extensions live in a third sibling workspace crate.

This is a Rust workspace with three members. See [`catgraph/README.md`](catgraph/README.md) for the slim F&S crate, [`catgraph-applied/docs/SEVEN-SKETCHES-AUDIT.md`](catgraph-applied/docs/SEVEN-SKETCHES-AUDIT.md) for the applied-CT coverage audit.

## Members

| Crate | Path | Purpose |
|---|---|---|
| `catgraph` v0.11.2 | `catgraph/` | Strict Fong-Spivak 2019 implementation |
| `catgraph-physics` v0.2.1 | `catgraph-physics/` | Wolfram-physics extensions: hypergraph DPO rewriting, multiway evolution, gauge theory, branchial spectral analysis |
| `catgraph-applied` v0.3.0 | `catgraph-applied/` | Applied CT extensions: `DecoratedCospan<F>` generic hypergraph categories (Def 6.75, Thm 6.77), Petri nets (with `HypergraphCategory` impl), wiring diagrams, E_n operads, Temperley-Lieb, linear combinations |

## Sibling repos

- [catgraph-surreal](https://github.com/tsondru/catgraph-surreal) — SurrealDB persistence for catgraph and catgraph-physics types
- [irreducible](https://github.com/tsondru/irreducible) — Gorard (2023) computational irreducibility framework

## Build

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -W clippy::pedantic
```

## Workflow

- Use `rust-analyzer` diagnostics before suggesting fixes
- Run `cargo check` after edits, `cargo test` after logic changes
- Prefer `cargo clippy -- -W clippy::pedantic` for lint passes

@.claude/refactor/workspace-overview.md
@.claude/refactor/current-plan.md
@.claude/refactor/session-state.md
@.claude/refactor/CLAUDE.local.md
