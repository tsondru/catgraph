# catgraph

Strict Rust implementation of [Fong & Spivak, *Hypergraph Categories* (2019)](https://arxiv.org/abs/1806.08304).

Cospans, spans, Frobenius algebras, hypergraph categories, compact closed structure, and the Theorem 1.2 equivalence. This crate tracks the F&S 2019 paper strictly — applied-CT extras and Wolfram-physics extensions live in sibling crates.

Originally based on a fork of [Cobord/Hypergraph](https://github.com/Cobord/Hypergraph), substantially rewritten to use source/target (cospan) semantics and implement the full F&S paper.

**v0.11.2 slim baseline** (v0.11.0 + explicit spider theorem tests). Rust 2024 edition, zero clippy pedantic warnings on lib, zero unsafe, criterion benchmarks.

## Component Index

| Module | Component | Purpose |
|--------|-----------|---------|
| `category.rs` | `HasIdentity`, `Composable`, `ComposableMutating` | Core composition traits |
| `cospan.rs` | `Cospan<Lambda>` | Morphisms in Cospan_Λ, pushout composition (union-find) |
| `span.rs` | `Span<Lambda>`, `Rel<Lambda>` | Pullback composition (dual), relation algebra |
| `named_cospan.rs` | `NamedCospan<Lambda, L, R>` | Port-labeled cospans for wiring-style composition |
| `monoidal.rs` | `Monoidal`, `SymmetricMonoidalMorphism`, `GenericMonoidalMorphism` | Tensor product, braiding, generic layered morphisms |
| `frobenius/` | `FrobeniusMorphism`, `MorphismSystem` | String diagram morphisms, DAG-based black-box interpretation |
| `compact_closed.rs` | `cup`, `cap`, `name`, `unname`, `compose_names_direct` | Self-dual compact closed structure (§3.1), Prop 3.3 literal form |
| `cospan_algebra.rs` | `CospanAlgebra`, `PartitionAlgebra`, `NameAlgebra`, `functor_induced_algebra_map` | Lax monoidal functors Cospan → Set (§2.1), Lemma 4.3 natural transformation |
| `hypergraph_category.rs` | `HypergraphCategory` | Frobenius generators η, ε, μ, δ with cup/cap (§2.3) |
| `hypergraph_functor.rs` | `HypergraphFunctor`, `RelabelingFunctor`, `CospanToFrobeniusFunctor` | Structure-preserving maps between hypergraph categories (§2.3) |
| `equivalence.rs` | `CospanAlgebraMorphism`, `comp_cospan`, `functor_from_algebra_morphism` | §4 equivalence Hyp_OF ≅ Cospan-Alg (Thm 1.2 per-Λ form), Lemma 4.9 io functor |
| `operadic.rs` | `Operadic` | Abstract operadic-substitution trait (Eq 6). Concrete impls live in `catgraph-applied` |
| `finset.rs` | `Permutation`, `Decomposition`, `OrderPresSurj`, `OrderPresInj` | Epi-mono factorization, order-preserving maps |

## Workspace siblings

**Workspace members** (share the same repo, co-evolve with catgraph):

| Crate | Purpose |
|-------|---------|
| [`catgraph-physics`](../catgraph-physics/) | Hypergraph DPO rewriting, multiway evolution, gauge theory, branchial spectral analysis |
| [`catgraph-applied`](../catgraph-applied/) | Petri nets, wiring diagrams, E_n operads, Temperley-Lieb, linear combinations — applied-CT extensions |

**Sibling repos** (separate git repos, depend on catgraph via git tag):

| Repo | Purpose |
|------|---------|
| [catgraph-surreal](https://github.com/tsondru/catgraph-surreal) | SurrealDB persistence for catgraph, catgraph-physics, and catgraph-applied types |
| [irreducible](https://github.com/tsondru/irreducible) | Computational irreducibility framework (Gorard 2023) |

## Fong-Spivak Feature Map

Features implementing structures from [Fong & Spivak, *Hypergraph Categories*](https://arxiv.org/abs/1806.08304). See [`docs/FONG-SPIVAK-AUDIT.md`](docs/FONG-SPIVAK-AUDIT.md) for the full per-section coverage audit.

| Paper Reference | Module | Summary |
|-----------------|--------|---------|
| Core (§1–2) | `cospan.rs` | `Cospan<Lambda>` — morphisms in Cospan_Λ, composition via pushout (union-find). |
| Core (§1–2) | `span.rs` | `Span<Lambda>` — dual of cospan, composition via pullback. Ex 2.15: Span/Rel. |
| Core | `category.rs` | `HasIdentity`, `Composable`, `ComposableMutating` traits for morphism composition. |
| Core | `monoidal.rs` | `Monoidal`, `SymmetricMonoidalMorphism` traits; tensor product and braiding. |
| Def 2.2 | `cospan_algebra.rs` | `CospanAlgebra` trait — lax monoidal functors Cospan_Λ → C. `PartitionAlgebra` (Ex 2.3, Prop 4.6: initial) and `NameAlgebra` (Prop 4.1). |
| Def 2.5 | `frobenius/` | `FrobeniusMorphism` — string diagram morphisms from the 4 Frobenius generators. `MorphismSystem` DAG for named composition. Ex 2.8: generators as cospans. |
| Def 2.12 | `hypergraph_category.rs` | `HypergraphCategory` trait — Frobenius generators (η, ε, μ, δ) with derived cup/cap. Prop 2.18 (strict case) implicitly satisfied. |
| Def 2.12, Eq 12 | `hypergraph_functor.rs` | `HypergraphFunctor` trait — structure-preserving maps. `RelabelingFunctor` (Thm 3.14: free functor). |
| Prop 3.1–3.4 | `compact_closed.rs` | Self-dual compact closed — cup/cap (Prop 3.1), name bijection (Prop 3.2), `compose_names_direct` realising the literal Prop 3.3 formula `(f̂ ⊗ ĝ) ; comp^Y_{X,Z}`, Prop 3.4 recovery tested by explicit `(id_X ⊗ f̂) ; (cap_X ⊗ id_Y)` construction. Zigzag identities (Eq 13). |
| Lemma 4.3 | `cospan_algebra.rs` | `functor_induced_algebra_map` lifts any `HypergraphFunctor` to a cospan-algebra morphism α: A_H → A_H'. |
| Lemma 4.9 | `equivalence.rs` | `functor_from_algebra_morphism` lifts a monoidal natural transformation α: A → B to the induced io hypergraph functor F_α: H_A → H_B. |
| Lemma 3.6, Prop 3.8 | `cospan_algebra.rs`, `hypergraph_functor.rs` | `cospan_to_frobenius` + `CospanToFrobeniusFunctor` — epi-mono decomposition into Frobenius generators. |
| **Thm 1.2** (per-Λ = Thm 4.13) | `equivalence.rs` | `CospanAlgebraMorphism<A>` (Lemma 4.8): cospan-algebra → hypergraph category. `comp_cospan` (Ex 3.5, Eq 32). Identity/Frobenius via Eq 33. Roundtrip: `Hyp_OF ≅ Cospan-Alg` per Λ. |

**Permanently deferred** (documented in [`docs/FONG-SPIVAK-AUDIT.md`](docs/FONG-SPIVAK-AUDIT.md) — require parametric Λ machinery or 2-category machinery beyond catgraph's current type system):

- Cross-Λ functoriality (Prop 2.1, Cor 3.13, Cor 3.15, Thm 3.14 universal property)
- Thm 1.1 strictification / coherence (Hyp ≃ Hyp_OF)
- §3.3 io/ff factorization (Lemma 3.19, Prop 3.20, Cor 3.21)
- Thm 4.16 global Grothendieck form (per-Λ Thm 4.13 suffices)
- LinRel examples (Ex 2.10, 2.11, 2.16, 2.20, 2.21, 4.14)

## Core: Cospans and Spans

Hyperedges connect **source sets** to **target sets** via typed middle sets:

```
    domain          middle         codomain
   [a, b]  ──left──▶ [x, y, z] ◀──right── [c, d]
```

An edge `[a,b] → [c,d]` means a→c, a→d, b→c, b→d (bipartite complete subgraph). This is distinct from path semantics where `[a,b,c,d]` means a→b→c→d.

| Type | Purpose |
|------|---------|
| `Cospan<Lambda>` | Morphisms in Cospan_Lambda. Composition via pushout (union-find, O(n·α(n))). |
| `NamedCospan<Lambda, L, R>` | Port-labeled cospans for wiring-style composition with named boundary nodes. |
| `Span<Lambda>` | Dual of cospan — composition via pullback. |
| `Rel<Lambda>` | Relations as jointly-injective spans. Full relation algebra. |

`Lambda` types the middle vertices — use `()` for untyped graphs.

## Examples

```bash
cargo run -p catgraph --example cospan
cargo run -p catgraph --example span
cargo run -p catgraph --example named_cospan
cargo run -p catgraph --example monoidal
cargo run -p catgraph --example finset
cargo run -p catgraph --example frobenius
cargo run -p catgraph --example hypergraph_category
cargo run -p catgraph --example compact_closed
cargo run -p catgraph --example cospan_algebra
cargo run -p catgraph --example hypergraph_functor
cargo run -p catgraph --example equivalence
```

Applied-CT examples (Petri net, wiring diagrams, operads, Temperley-Lieb) now live in `catgraph-applied/examples/`.

## Testing

```bash
cargo test -p catgraph
cargo test -p catgraph --examples
cargo clippy -p catgraph -- -W clippy::pedantic
```

## Dependencies

- `rustworkx-core` — graph algorithms
- `itertools` — iterator utilities
- `either` — Left/Right sum type for bipartite node types
- `num` — numeric traits (One, Zero)
- `permutations` — permutation type for symmetric monoidal
- `union-find` — QuickUnionUf for pushout composition
- `rayon` — data parallelism with adaptive thresholds (rayon 1.12 `with_min_len`)
- `log` — warning messages
- `rand` — random number generation
- `rust_decimal` — exact decimal arithmetic
- `thiserror` — structured error types
- Dev: `env_logger`, `proptest`, `criterion`

## Usage

```toml
[dependencies]
catgraph = { git = "https://github.com/tsondru/catgraph", tag = "v0.11.0" }
```

## References

- [Fong & Spivak, *Hypergraph Categories* (2019)](https://arxiv.org/abs/1806.08304) — primary theoretical foundation
- [Spivak, *The Operad of Wiring Diagrams* (2013)](https://arxiv.org/abs/1305.0297)

## License

[MIT](LICENSE)
