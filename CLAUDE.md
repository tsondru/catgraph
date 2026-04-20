# catgraph workspace

Category-theoretic graph structures in Rust. The core [`catgraph`](catgraph/) crate (v0.11.4) is a strict implementation of [Fong & Spivak, *Hypergraph Categories* (2019)](https://arxiv.org/abs/1806.08304). [`catgraph-applied`](catgraph-applied/) (v0.4.0) anchors applied-CT extensions to [Fong & Spivak, *Seven Sketches in Compositionality* (2018)](https://arxiv.org/abs/1803.05316). Wolfram-physics extensions live in a third sibling workspace crate.

This is a Rust workspace with three members. See [`catgraph/README.md`](catgraph/README.md) for the slim F&S crate, [`catgraph-applied/docs/SEVEN-SKETCHES-AUDIT.md`](catgraph-applied/docs/SEVEN-SKETCHES-AUDIT.md) for the applied-CT coverage audit.

## Members

| Crate | Path | Purpose |
|---|---|---|
| `catgraph` v0.11.3 | `catgraph/` | Strict Fong-Spivak 2019 implementation |
| `catgraph-physics` v0.2.1 | `catgraph-physics/` | Wolfram-physics extensions: hypergraph DPO rewriting, multiway evolution, gauge theory, branchial spectral analysis |
| `catgraph-applied` v0.4.0 | `catgraph-applied/` | Applied CT extensions: `DecoratedCospan<F>` (Def 6.75, Thm 6.77; v0.3.x), Petri nets (with `HypergraphCategory` impl), wiring diagrams, E_n operads, Temperley-Lieb, linear combinations, plus **v0.4.0 Tier 2**: `Prop` + `Free(G)` (Def 5.2, 5.25), `OperadAlgebra<O>` with `CircAlgebra` (Def 6.99, Ex 6.100), `OperadFunctor` with canonical `E₁ ↪ E₂` (Rough Def 6.98) |

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

## Release procedure

1. **CHANGELOG-first.** Before tagging any crate, update its `CHANGELOG.md`: promote `[Unreleased]` entries into a new `[X.Y.Z] - YYYY-MM-DD` section, add a new empty `[Unreleased]` header, and update the link references at the bottom of the file. The CHANGELOG is the source of truth for "what shipped in this version" — not commit messages, not README blurbs.
2. **Per-crate versioning.** Each workspace crate versions independently (`catgraph`, `catgraph-applied`, `catgraph-physics`). Bump only the crate's `Cargo.toml` version when its scope changed.
3. **Dual-tag when crates co-release.** When a catgraph change is consumed by catgraph-applied in the same logical release (e.g., v0.11.3 + catgraph-applied-v0.3.1), tag both at the same commit SHA. Tag scheme: `v<ver>` for catgraph, `<crate>-v<ver>` for sibling crates.
4. **Verify before tagging.** `cargo test --workspace`, `cargo clippy --workspace --lib --tests`, `cargo test --workspace --examples`, `cargo doc --workspace --no-deps` — all clean, no new warnings attributable to the release.
5. **Session-state is workspace-level only.** Track in-flight work in `.claude/refactor/session-state.md` and `.claude/refactor/current-plan.md`. Do not create per-crate `<crate>-session-state.md` files — CHANGELOGs carry shipped-work history.
6. **Roadmap + audit separation.** Forward work lives in `.claude/docs/ROADMAP.md` + per-crate audit docs' "Tier" tables. CHANGELOGs carry release history. Audit docs stay paper-mapping only; no release-history duplication inside them.

@.claude/docs/workspace-overview.md
@.claude/refactor/current-plan.md
@.claude/refactor/session-state.md
@.claude/refactor/CLAUDE.local.md
