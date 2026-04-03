# catgraph

Category-theoretic graph structures in Rust: cospans, spans, wiring diagrams, Frobenius algebras, E_n operads, bifunctors, adjunctions, monoidal coherence verification, and morphisms in (symmetric) monoidal categories, with SurrealDB persistence.

Originally based on a fork of [Cobord/Hypergraph](https://github.com/Cobord/Hypergraph), substantially rewritten to use source/target (cospan) semantics, add relation algebra, Temperley-Lieb/Brauer diagrams, E_n operads, morphism systems, and SurrealDB persistence.

515 tests (including 8 proptest properties), zero clippy warnings, criterion benchmarks. Rust 2024 edition.

## What catgraph implements

catgraph is an **applied category theory** library for compositional systems — specifically [Fong-Spivak](https://arxiv.org/pdf/1806.08304.pdf)-style string diagrams and [cospans](https://en.wikipedia.org/wiki/Span_(category_theory)) with source/target hypergraph semantics. It is not a general category theory library.

### Core: Cospans and Spans

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
| `Rel<Lambda>` | Relations as jointly-injective spans. Full relation algebra (see below). |

`Lambda` types the middle vertices — use `()` for untyped graphs.

### Category Traits

Morphisms in catgraph implement compositional traits:

```rust
pub trait HasIdentity<T>: Sized {
    fn identity(on_this: &T) -> Self;
}

pub trait Composable<T: Eq>: Sized {
    fn compose(&self, other: &Self) -> Result<Self, CatgraphError>;
    fn domain(&self) -> T;
    fn codomain(&self) -> T;
}

pub trait Monoidal {
    fn monoidal(&mut self, other: Self);  // tensor product
}

pub trait SymmetricMonoidalMorphism<T: Eq>: Composable<Vec<T>> + Monoidal {
    fn from_permutation(p: Permutation, types: &[T], types_as_on_domain: bool) -> Result<Self, CatgraphError>;
    fn permute_side(&mut self, p: &Permutation, of_codomain: bool);
}
```

### Relation Algebra (`Rel`)

`Rel<Lambda>` wraps a `Span<Lambda>` with the joint injectivity invariant, providing:

```rust
// Construction
Rel::new(span) -> Result<Self, CatgraphError>  // validates joint injectivity
Rel::new_unchecked(span) -> Self               // trusts caller

// Set operations
rel.union(&other)?, rel.intersection(&other)?, rel.complement()?
rel.subsumes(&other)? -> bool

// Properties (require homogeneous relation: domain == codomain)
rel.is_reflexive(), rel.is_symmetric(), rel.is_antisymmetric(), rel.is_transitive()
rel.is_equivalence_rel(), rel.is_partial_order(), rel.is_irreflexive()
rel.is_homogeneous() -> bool
```

### Frobenius Algebra

Morphisms built from the four distinguished morphisms of a Frobenius object (multiplication, comultiplication, unit, counit) plus braiding and black boxes. The black boxes are labelled and can be interpreted via a user-supplied function.

**MorphismSystem** — a DAG-based framework for named morphism collections with acyclic black-box substitution:

```rust
let mut sys = MorphismSystem::new("circuit".to_string());
sys.add_definition_simple("resistor", resistor_morphism)?;
sys.add_definition_simple("capacitor", capacitor_morphism)?;
sys.add_definition_composite("rc_filter", rc_filter_template)?;  // references resistor + capacitor
let resolved = sys.fill_black_boxes(None)?;  // topological resolution
```

Cycle detection prevents circular definitions. The `Contains` and `InterpretableMorphism` traits enable custom interpretation.

### Brauer / Temperley-Lieb Algebra

[Brauer algebra](https://en.wikipedia.org/wiki/Brauer_algebra) and Temperley-Lieb diagrams over an arbitrary ground ring via `LinearCombination<BrauerMorphism>`.

```rust
let gens = BrauerMorphism::<i64>::temperley_lieb_gens(5);  // e_0 .. e_3
let sym = BrauerMorphism::<i64>::symmetric_alg_gens(5);     // s_0 .. s_3

// TL relations hold:
// e_i * e_i = δ * e_i  (idempotent up to delta)
// s_i * s_i = id        (involution)
// e_i * s_i = e_i       (absorption)
// s_i * s_{i+1} * s_i = s_{i+1} * s_i * s_{i+1}  (braid/Yang-Baxter)
```

### E_n Operads

[Little cubes operads](https://ncatlab.org/nlab/show/little+cubes+operad) with fallible constructors and epsilon tolerance for floating-point boundaries:

| Operad | Objects | Operations |
|--------|---------|------------|
| `E1` | Configurations of intervals in [0,1] | Operadic substitution, coalescence, monoid homomorphism |
| `E2` | Configurations of disks in the unit disk | Operadic substitution, coalescence, `from_e1_config` embedding |

```rust
let e1 = E1::new(vec![(0.0, 0.3), (0.5, 0.8)], true)?;  // overlap_check=true
let e2 = E2::from_e1_config(e1, |i| format!("disk_{i}"));  // embed intervals as disks on x-axis
```

### Wiring Diagrams

An operad built on `NamedCospan` implementing the [wiring diagram operad](https://arxiv.org/abs/1305.0297) for compositional system modeling.

![Wiring Diagram from Spivak 13](./assets/wiring.png)

### Finite Sets

`finset.rs` provides morphisms between finite sets:
- `Permutation` — via the `permutations` crate
- `OrderPresSurj` / `OrderPresInj` — order-preserving surjections/injections
- `Decomposition` — epi-mono factorization (every morphism = surjection ∘ injection)

### Linear Combinations

`LinearCombination<T, Basis>` — formal linear combinations over a ring. Supports ring axioms (add, mul, scalar mul) with rayon-parallel multiplication above 32 terms. Used as the coefficient structure for Brauer algebra diagrams.

## SurrealDB Persistence

The `catgraph-surreal` workspace member provides typed persistence for catgraph structures in [SurrealDB](https://surrealdb.com/) with two coexisting layers:

**V1 (embedded arrays)** — O(1) reconstruction via `CospanStore`, `NamedCospanStore`, `SpanStore`. Each n-ary hyperedge is a single record with embedded arrays encoding the structural maps.

**V2 (RELATE-based graph)** — Graph-native persistence with `NodeStore`, `EdgeStore`, `HyperedgeStore`, and `QueryHelper`. Supports edge properties, multi-hop traversal, hub-node reification for n-ary hyperedges, record references with `ON DELETE UNSET`, and computed provenance fields.

```rust
use catgraph_surreal::{init_schema, init_schema_v2};
use catgraph_surreal::hyperedge_store::HyperedgeStore;

let db = Surreal::new::<Mem>(()).await?;
db.use_ns("test").use_db("test").await?;
init_schema(&db).await?;
init_schema_v2(&db).await?;

let store = HyperedgeStore::new(&db);
let hub_id = store.decompose_cospan(&cospan, "reaction", props, |c| c.to_string()).await?;
let reconstructed: Cospan<char> = store.reconstruct_cospan(&hub_id).await?;
```

108 integration tests cover V1 roundtrips, V2 CRUD/traversal, provenance, and domain-specific use cases (code graphs, chemical reactions, dataflow pipelines, API orchestration, circuit design).

## Examples

Standalone examples in `examples/` demonstrate each module's pub API:

```bash
cargo run --example interval            # DiscreteInterval + ParallelIntervals
cargo run --example complexity          # StepCount + Complexity trait
cargo run --example computation_state   # ComputationState lifecycle
cargo run --example adjunction          # ZPrimeOps + AdjunctionVerification
cargo run --example bifunctor           # TensorProduct + IntervalTransform + verify_*
cargo run --example coherence           # CoherenceVerification + DifferentialCoherence
cargo run --example stokes              # TemporalComplex + ConservationResult
```

## Testing

```bash
cargo test --workspace        # 515 tests (407 catgraph + 108 bridge), 1 ignored
cargo test                    # catgraph-only (407: 263 unit + 144 integration)
cargo test -p catgraph-surreal # bridge crate (108: 10 unit + 98 integration)
cargo clippy                  # zero warnings
```

Integration test suites:

| File | Tests | What it covers |
|------|-------|---------------|
| `composition_laws` | 17 | Associativity, identity, empty/large boundaries |
| `pushout_correctness` | 9 | Union-find pushout, wire merging, determinism |
| `relation_algebra` | 21 | Rel API, equivalence relations, partial orders, set operations |
| `frobenius_laws` | 8 | Braiding, spider fusion, unit/counit, monoidal |
| `monoidal_structure` | 6 | Tensor associativity/unit, braiding, permute_side |
| `cross_type_interactions` | 6 | NamedCospan ports, to_graph, ring axioms |
| `morphism_system` | 8 | DAG resolution, cycle detection, multi-level fill |
| `operad_boundary` | 17 | E1/E2 epsilon boundaries, embedding, substitution, coalescence, min_closeness |
| `temperley_lieb` | 10 | TL/symmetric generators, braid relation, monoidal |
| `property_laws` | 8 | Proptest: identity, associativity, dagger involution, monoidal |
| `wiring_diagram` | 14 | Operadic substitution, boundary mutations, map, sequential composition |
| `mutation_workflows` | 20 | Cospan/Span add/delete/connect/map then compose, identity flags |

## Parallelization

The library uses rayon for parallel computation with adaptive thresholds:

| Module | Parallelized Operation | Threshold |
|--------|------------------------|-----------|
| `linear_combination.rs` | `Mul` impl, `linear_combine` | 32 terms |
| `temperley_lieb.rs` | `non_crossing` checks | 8 elements |
| `named_cospan.rs` | `find_nodes_by_name_predicate` | 256 elements |
| `frobenius/operations.rs` | `hflip` block mutations | 64 blocks |

All parallelism is rayon-based (CPU-bound). For tokio integration, use **tokio-rayon** (not `spawn_blocking`).

## Benchmarks

Criterion benchmarks in `benches/` cover core operations and rayon threshold validation:

```bash
cargo bench                              # run all benchmarks
cargo bench --bench pushout              # cospan pushout composition (sizes 4–1024)
cargo bench --bench pullback             # span pullback composition (sizes 4–1024)
cargo bench --bench interval             # interval composition, tensor, direct sum
cargo bench --bench rayon_thresholds     # validate rayon parallel thresholds
```

HTML reports are generated in `target/criterion/`. For profiling:

```bash
cargo install flamegraph
cargo flamegraph --bench pushout -- --bench "pushout_compose/1024"
```

## Dependencies

### catgraph (core)
- `petgraph` — graph data structures (StableDiGraph, toposort, connectivity)
- `itertools` — iterator utilities
- `either` — Left/Right sum type for bipartite node types
- `num` — numeric traits (One, Zero)
- `permutations` — permutation type for symmetric monoidal
- `union-find` — QuickUnionUf for pushout composition
- `rayon` — data parallelism with adaptive thresholds
- `log` — warning messages
- `thiserror` — structured error types
- Dev: `env_logger`, `proptest`, `rand`, `criterion`

### catgraph-surreal (bridge)
- `surrealdb` 3.0.5 (kv-mem) — embedded SurrealDB
- `surrealdb-types` 3.0.5 — SurrealValue derive macro
- `serde` + `serde_json` — JSON serialization for Lambda labels
- `tokio` — async runtime
- `thiserror` — error type derivation

## Usage

```toml
[dependencies]
catgraph = { git = "https://github.com/tsondru/catgraph" }
catgraph-surreal = { git = "https://github.com/tsondru/catgraph" }  # optional
```

```rust
use catgraph::cospan::Cospan;
use catgraph::span::{Span, Rel};
use catgraph::named_cospan::NamedCospan;
use catgraph::frobenius::MorphismSystem;
use catgraph::temperley_lieb::BrauerMorphism;
use catgraph::e1_operad::E1;
use catgraph::e2_operad::E2;
use catgraph::errors::CatgraphError;
use catgraph::category::{Composable, HasIdentity};
use catgraph::monoidal::{Monoidal, SymmetricMonoidalMorphism};
```

## Contributors

- [tsondru](https://github.com/tsondru)
- [Claude](https://claude.ai) (Anthropic)

## Acknowledgments

This project originated as a fork of [Cobord/Hypergraph](https://github.com/Cobord/Hypergraph).

## References

- [Fong-Spivak: Hypergraph Categories](https://arxiv.org/pdf/1806.08304.pdf) — cospan semantics
- [Span and Cospan (Wikipedia)](https://en.wikipedia.org/wiki/Span_(category_theory))
- [E_n Operad (nLab)](https://ncatlab.org/nlab/show/little+cubes+operad)
- [Brauer Algebra (Wikipedia)](https://en.wikipedia.org/wiki/Brauer_algebra)
- [Wiring Diagrams (Spivak 2013)](https://arxiv.org/abs/1305.0297)

## License

[MIT](LICENSE)
