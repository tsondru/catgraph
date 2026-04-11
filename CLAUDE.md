# catgraph workspace

Category-theoretic graph structures in Rust. Implements [Fong & Spivak, *Hypergraph Categories* (2019)](https://arxiv.org/abs/1806.08304).

This is a Rust workspace. See [`catgraph/README.md`](catgraph/README.md) for the slim F&S crate.

## Members

| Crate | Path | Purpose |
|---|---|---|
| `catgraph` | `catgraph/` | Strict Fong-Spivak 2019 implementation |

## Sibling repos

- [catgraph-surreal](https://github.com/tsondru/catgraph-surreal) — SurrealDB persistence for catgraph types
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
