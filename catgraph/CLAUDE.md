# catgraph (F&S 2019 crate, v0.11.0 slim baseline)

Strict implementation of [Fong & Spivak, *Hypergraph Categories* (2019)](https://arxiv.org/abs/1806.08304). v0.11.0 is the slim F&S baseline release — applied-CT extras now live in `catgraph-applied`, Wolfram-physics extensions in `catgraph-physics`.

## Scope

This crate implements the paper's core definitions and main theorems:

- `Cospan<Λ>` with pushout composition (§1)
- `FrobeniusMorphism` (§2.2 Def 2.5)
- `CospanAlgebra` (§2.1 Def 2.2) with `PartitionAlgebra` (Ex 2.3) and `NameAlgebra` (§4.1)
- `HypergraphCategory`, `HypergraphFunctor` (§2.3 Def 2.12, Eq 12)
- Self-dual compact closed structure (§3.1 Props 3.1–3.4)
- `CospanToFrobeniusFunctor` (§3.2 Prop 3.8)
- `CospanAlgebraMorphism` and the §4 equivalence (Thm 1.2 / Thm 4.13)

See [`docs/FONG-SPIVAK-AUDIT.md`](docs/FONG-SPIVAK-AUDIT.md) for section-by-section paper coverage.

**Permanently deferred** (require parametric Λ machinery beyond catgraph's type system):
- Cross-Λ functoriality (Prop 2.1, Cor 3.13, Cor 3.15)
- Theorem 1.1 strictification (§3.4 Thm 3.22)
- Theorem 4.16 global Grothendieck form
- §3.3 io/ff factorization
- LinRel examples (2.10, 2.11, 2.16, 2.20, 2.21, 4.14)

**Out of scope** (delegated):
- Hypergraph DPO rewriting, multiway evolution, gauge theory, branchial spectral analysis → `catgraph-physics` (workspace sibling, v0.2.0)
- Petri nets, wiring diagrams, E_n operads, Temperley-Lieb, linear combinations → `catgraph-applied` (workspace sibling, v0.1.0)
- Persistence → [catgraph-surreal](https://github.com/tsondru/catgraph-surreal)
- Computational irreducibility → [irreducible](https://github.com/tsondru/irreducible)

## Core semantics: source/target (cospan)

Hyperedges connect **source sets** to **target sets**. An edge `[a,b] → [c,d]` creates the bipartite complete subgraph `a→c, a→d, b→c, b→d`. This differs from path-based hypergraph libraries (e.g., yamafaktory/hypergraph) where `[a,b,c,d]` means a sequential chain `a→b→c→d`.

## Build

```sh
cargo test -p catgraph
cargo test -p catgraph --examples
cargo bench -p catgraph --no-run
```

## Type constraints

- `Lambda: Sized + Eq + Copy + Debug` for most types
- Names in `NamedCospan` need `Eq + Clone` (and `Hash` for validation)
- Group elements need `One + MulAssign + Eq + Clone`

## Clippy preferences (Rust 2024 edition)

- Use `matches!` macro instead of match expressions returning bool
- Use `.is_multiple_of()` instead of `% n == 0`
- Collapse nested `if let` with `&&` (let chains)

@../.claude/refactor/catgraph-architecture.md
@../.claude/refactor/fs-coverage-detail.md
@../.claude/refactor/catgraph-session-state.md
@../.claude/refactor/catgraph-CLAUDE.local.md
