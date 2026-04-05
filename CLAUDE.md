# catgraph - Category-Based Cospan Graph Tools

## Project Overview

**catgraph** implements category-theoretic graph structures in Rust, focusing on source/target (cospan) semantics for hypergraphs. This is distinct from path-based hypergraph semantics used by libraries like yamafaktory/hypergraph.

Originally based on a fork of [Cobord/Hypergraph](https://github.com/Cobord/Hypergraph), substantially rewritten.

### Core Semantics: Source/Target (Cospan)

In catgraph, hyperedges connect **source sets** to **target sets**:
- An edge `[a,b] → [c,d]` creates connections: a→c, a→d, b→c, b→d (bipartite complete subgraph)
- Based on category theory (cospans)
- Uses petgraph for underlying graph representation

This differs from path semantics where `[a,b,c,d]` means a→b→c→d (sequential chain).

## Workspace Structure

```
catgraph/                           # Workspace root
├── Cargo.toml                      # Workspace: members = [".", "catgraph-surreal"]
├── src/
│   ├── errors.rs                   # CatgraphError with thiserror (..., PetriNet, FinSet)
│   ├── category.rs                 # Core traits: HasIdentity, Composable, ComposableMutating
│   ├── monoidal.rs                 # Monoidal + symmetric monoidal traits, GenericMonoidalMorphism
│   ├── operadic.rs                 # Operadic trait for substitution
│   │
│   ├── cospan.rs                   # Core cospan implementation over Lambda-typed sets
│   ├── named_cospan.rs             # Cospans with named boundary nodes
│   ├── span.rs                     # Span and Rel (relations) implementations
│   │
│   ├── frobenius/                  # Frobenius algebra (split from single 2254-LOC file)
│   │   ├── mod.rs                  # Re-exports preserving public API
│   │   ├── morphism_system.rs      # Contains, InterpretableMorphism, MorphismSystem (DAG resolution)
│   │   ├── operations.rs           # FrobeniusOperation, FrobeniusBlock, FrobeniusLayer, FrobeniusMorphism
│   │   └── trait_impl.rs           # Frobenius trait + blanket InterpretableMorphism impl
│   ├── temperley_lieb.rs           # Temperley-Lieb / Brauer algebra
│   │
│   ├── wiring_diagram.rs           # Wiring diagram operad built on cospans
│   │
│   ├── petri_net.rs                # PetriNet, Transition (Decimal weights), Marking, firing, reachability, cospan bridge
│   │
│   ├── e1_operad.rs                # E1 operad (intervals in [0,1])
│   ├── e2_operad.rs                # E2 operad (disks in unit disk)
│   │
│   ├── finset.rs                   # Finite set morphisms, permutations, epi-mono factorization
│   │
│   ├── interval.rs                 # DiscreteInterval, ParallelIntervals (extracted from irreducible)
│   ├── complexity.rs               # Complexity trait, StepCount (extracted from irreducible)
│   ├── computation_state.rs        # ComputationState (extracted from irreducible)
│   ├── adjunction.rs               # ZPrimeOps, AdjunctionVerification, AdjunctionIrreducibility (extracted from irreducible)
│   ├── bifunctor.rs                # TensorProduct, IntervalTransform, tensor_bimap/first/second (extracted from irreducible)
│   ├── coherence.rs                # Monoidal coherence verifiers: associator, unitors, braiding (extracted from irreducible)
│   ├── stokes.rs                   # TemporalComplex, ConservationResult, StokesError (extracted from irreducible)
│   │
│   ├── linear_combination.rs       # Linear combinations over rings
│   ├── utils.rs                    # Permutation utilities, helpers
│   └── lib.rs                      # Library exports (all modules pub)
│
├── examples/                       # Standalone API examples (one per module)
│   ├── interval.rs                 # DiscreteInterval + ParallelIntervals
│   ├── complexity.rs               # StepCount + Complexity trait
│   ├── computation_state.rs        # ComputationState lifecycle
│   ├── adjunction.rs               # ZPrimeOps + AdjunctionVerification
│   ├── bifunctor.rs                # TensorProduct + IntervalTransform + verify_*
│   ├── coherence.rs                # CoherenceVerification + DifferentialCoherence
│   ├── stokes.rs                   # TemporalComplex + ConservationResult
│   ├── cospan.rs                   # Cospan construction, composition, monoidal
│   ├── span.rs                     # Span, Rel algebra
│   ├── named_cospan.rs             # Port-labeled cospans
│   ├── monoidal.rs                 # Tensor product, braiding, GenericMonoidalMorphism
│   ├── finset.rs                   # Permutations, epi-mono factorization
│   ├── frobenius.rs                # String diagrams, MorphismSystem DAG
│   ├── e1_operad.rs                # Little intervals operad
│   ├── e2_operad.rs                # Little disks operad
│   ├── wiring_diagram.rs           # Wiring diagram operad
│   ├── temperley_lieb.rs           # TL/Brauer generators, braid relation
│   ├── linear_combination.rs       # Linear combinations over morphisms
│   └── petri_net.rs                # Petri net firing, reachability, composition
│
├── benches/                        # Criterion benchmarks
│   ├── pushout.rs                  # Cospan::compose at sizes 4–1024
│   ├── pullback.rs                 # Span::compose at sizes 4–1024
│   ├── interval.rs                 # DiscreteInterval + ParallelIntervals ops
│   └── rayon_thresholds.rs         # Rayon threshold validation (4 operations)
│
├── tests/                          # Integration tests (public API only)
│   ├── common/mod.rs               # Shared test helpers: cospan_eq, span_eq, assert_*_eq
│   ├── composition_laws.rs         # 17 tests: associativity, identity, empty/large boundaries
│   ├── pushout_correctness.rs      # 9 tests: union-find pushout, wire merging, determinism
│   ├── relation_algebra.rs         # 21 tests: Rel API, dagger involution, span composition, equivalence/partial order
│   ├── frobenius_laws.rs           # 8 tests: braiding, spider fusion, unit/counit, monoidal
│   ├── monoidal_structure.rs       # 6 tests: tensor associativity/unit, braiding, permute_side
│   ├── cross_type_interactions.rs  # 6 tests: NamedCospan ports, to_graph, LinearCombination ring
│   ├── morphism_system.rs          # 8 tests: DAG resolution, cycle detection, multi-level fill
│   ├── operad_boundary.rs          # 17 tests: E1/E2 epsilon boundaries, embedding, substitution, coalescence, min_closeness
│   ├── temperley_lieb.rs           # 10 tests: TL/symmetric generators, braid relation, monoidal
│   ├── property_laws.rs            # 8 tests: proptest algebraic laws (identity, associativity, dagger, monoidal)
│   ├── wiring_diagram.rs           # 14 tests: operadic substitution, boundary mutations, map, sequential composition
│   ├── mutation_workflows.rs       # 20 tests: Cospan/Span add/delete/connect/map then compose, identity flags
│   └── petri_net.rs                # 8 tests: chemical reactions, reachability, composition, cospan roundtrip
│
└── catgraph-surreal/               # SurrealDB persistence bridge crate
    ├── Cargo.toml                  # Depends on catgraph + surrealdb 3.0.5 (kv-mem)
    ├── src/
    │   ├── lib.rs                  # init_schema() + init_schema_v2() + module re-exports
    │   ├── error.rs                # PersistError enum (thiserror)
    │   ├── persist.rs              # Persistable trait + impls (char, (), u32, i32, i64, u64, String, Decimal)
    │   ├── schema.rs               # V1 SurrealQL DDL (embedded arrays)
    │   ├── schema_v2.rs            # V2 SurrealQL DDL (RELATE-based graph tables, FTS, HNSW, Petri net)
    │   ├── types.rs                # V1 record types (SurrealValue derives)
    │   ├── types_v2.rs             # V2 record types (GraphNode, GraphEdge, HyperedgeHub, PetriNet, Marking)
    │   ├── cospan_store.rs         # V1 CospanStore: save/load/delete/list
    │   ├── named_cospan_store.rs   # V1 NamedCospanStore (composes with CospanStore)
    │   ├── span_store.rs           # V1 SpanStore: save/load/delete/list
    │   ├── node_store.rs           # V2 NodeStore: CRUD for graph_node records
    │   ├── edge_store.rs           # V2 EdgeStore: RELATE edges, traversal
    │   ├── hyperedge/              # V2 HyperedgeStore (split from single 738-LOC file)
    │   │   ├── mod.rs              # HyperedgeStore struct, hub CRUD, private helpers
    │   │   ├── decompose.rs        # decompose_cospan/span/named_cospan, atomic, retry
    │   │   ├── reconstruct.rs      # reconstruct_cospan/span, sources/targets
    │   │   └── provenance.rs       # composition provenance tracking
    │   ├── petri_net_store.rs       # V2 PetriNetStore: save/load/delete topology + markings
    │   ├── wiring_store.rs         # V2 WiringDiagramStore: decompose/reconstruct via hub-node
    │   ├── fingerprint.rs          # Structural fingerprint computation (petgraph) + HNSW search
    │   └── query.rs                # V2 QueryHelper: neighbors, reachable, shortest_path, collect
    └── tests/
        ├── v1_cospan_roundtrip.rs          # 9 tests: V1 char/unit roundtrip, identity, compose-persist
        ├── v1_named_cospan_roundtrip.rs    # 5 tests: V1 port name preservation, record references
        ├── v1_span_roundtrip.rs            # 8 tests: V1 span/dagger roundtrip, identity flags
        ├── v1_v2_coexistence.rs            # 6 tests: span/named cospan, table/delete isolation
        ├── v2_node_edge_crud.rs            # 23 tests: V2 node/edge/hyperedge CRUD, traversal
        ├── v2_atomic_decompose.rs          # 8 tests: atomic vs non-atomic decompose
        ├── v2_span_decompose.rs            # 5 tests: V2 span decompose/reconstruct
        ├── v2_provenance.rs                # 11 tests: provenance + schema features (REFERENCE, ON DELETE UNSET, COMPUTED)
        ├── domain_api_orchestration.rs     # 4 tests: API orchestration (hub properties)
        ├── v2_petri_net.rs                 # 6 tests: PetriNet store roundtrip, marking persistence
        ├── v2_wiring_diagram.rs            # 8 tests: WiringDiagram store roundtrip, port metadata
        ├── v2_graph_recursion.rs           # 10 tests: shortest_path, collect_reachable, depth limits
        ├── v2_fingerprint_search.rs        # 5 tests: fingerprint compute/store, HNSW similarity
        ├── v2_schema_modernization.rs      # 2 tests: FTS node name search
        ├── domain_chemical_reactions.rs    # 5 tests: chemical reactions (Cospan hyperedges)
        ├── domain_circuit_design.rs        # 5 tests: cascaded logic gates, shared nodes
        ├── domain_code_analysis.rs         # 5 tests: code graph (pairwise, multi-hop)
        └── domain_dataflow_pipeline.rs     # 4 tests: NamedCospan dataflow
```

## Key Types and Traits

### Category Traits (`category.rs`)

```rust
pub trait HasIdentity<T>: Sized {
    fn identity(on_this: &T) -> Self;
}

pub trait Composable<T: Eq>: Sized {
    fn compose(&self, other: &Self) -> Result<Self, CatgraphError>;
    fn domain(&self) -> T;
    fn codomain(&self) -> T;
}

pub trait ComposableMutating<T: Eq>: Sized {
    fn compose(&mut self, other: Self) -> Result<(), CatgraphError>;
    // ... domain, codomain
}
```

### Monoidal + Symmetric Monoidal (`monoidal.rs`)

```rust
pub trait Monoidal {
    fn monoidal(&mut self, other: Self);
}

pub trait SymmetricMonoidalMorphism<T: Eq>: Composable<Vec<T>> + Monoidal {
    fn from_permutation(p: Permutation, types: &[T], types_as_on_domain: bool) -> Result<Self, CatgraphError>;
    fn permute_side(&mut self, p: &Permutation, of_codomain: bool);
}
```

### Error Handling (`errors.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CatgraphError {
    CompositionSizeMismatch { expected: usize, actual: usize },
    CompositionLabelMismatch { index: usize, expected: String, actual: String },
    Composition { message: String },
    Interpret { context: String },
    Operadic { message: String },
    Relation { message: String },
}
```

### Cospan (`cospan.rs`)

```rust
pub struct Cospan<Lambda> {
    left: Vec<MiddleIndex>,   // domain → middle
    right: Vec<MiddleIndex>,  // codomain → middle
    middle: Vec<Lambda>,      // typed middle set
}
```

- `Lambda` types the vertices (use `()` for untyped)
- Composition via pushout (union-find, O(n * alpha(n)))
- Supports `Monoidal`, `Composable`, `SymmetricMonoidalMorphism`
- Public accessors: `left_to_middle()`, `right_to_middle()`, `middle()`, `is_left_identity()`, `is_right_identity()`

### Named Cospan (`named_cospan.rs`)

```rust
pub struct NamedCospan<Lambda, LeftPortName, RightPortName> {
    cospan: Cospan<Lambda>,
    left_names: Vec<LeftPortName>,
    right_names: Vec<RightPortName>,
}
```

- Public accessors: `cospan()`, `left_names()`, `right_names()`

### Span/Rel (`span.rs`)

```rust
pub struct Span<Lambda> { ... }
pub struct Rel<Lambda>(Span<Lambda>);
```

- Public accessors: `left()`, `right()`, `middle_pairs()`, `is_left_identity()`, `is_right_identity()`
- `Rel::as_span()` for bridge crate access
- Relations with: `is_reflexive`, `is_symmetric`, `is_antisymmetric`, `is_transitive`, `is_equivalence_rel`, `is_partial_order`, `subsumes` (→ `Result<bool>`), `intersection` / `union` / `complement` (→ `Result<Self>`).

### Frobenius (`frobenius/`)

Split into submodules: `morphism_system.rs` (Contains, InterpretableMorphism, MorphismSystem DAG), `operations.rs` (FrobeniusOperation, FrobeniusBlock, FrobeniusLayer, FrobeniusMorphism + all trait impls), `trait_impl.rs` (Frobenius trait + blanket impl). Public API unchanged via re-exports in `mod.rs`.

## SurrealDB Persistence (`catgraph-surreal`)

Bridge crate for persisting catgraph structures to SurrealDB. Two layers coexist on different tables:

### V1: Embedded Arrays (O(1) reconstruction)

Each n-ary hyperedge stored as a single record with embedded arrays encoding the structural maps.

- **`Persistable`** trait — JSON serialization for Lambda types without requiring serde on catgraph core. Impls for `char`, `()`, `u32`, `i32`, `i64`, `u64`, `String`.
- **`CospanStore`** / **`NamedCospanStore`** / **`SpanStore`** — typed async CRUD (save/load/delete/list).
- Tables: `cospan`, `named_cospan`, `span`.

### V2: RELATE-Based Graph Persistence

Graph-native persistence with first-class nodes, pairwise edges, and hub-node reification for n-ary hyperedges. Supports edge properties, graph traversal, and SurrealDB-native queries.

- **`NodeStore`** — CRUD for `graph_node` records (name, kind, labels, properties).
- **`EdgeStore`** — `RELATE`-based pairwise edges with traversal (outbound/inbound/between).
- **`HyperedgeStore`** — Decompose `Cospan`/`Span`/`NamedCospan` into hub-node reification pattern (`hyperedge_hub` + `source_of`/`target_of` edges). Reconstruct `Cospan<Lambda>` from hub.
- **`PetriNetStore`** — Native Petri net persistence: save/load/delete topology + marking snapshots.
- **`WiringDiagramStore`** — WiringDiagram V2 persistence via hub-node reification with port metadata.
- **`FingerprintEngine`** — Structural fingerprint computation (petgraph) + HNSW similarity search.
- **`QueryHelper`** — Graph traversal: `outbound_neighbors`, `inbound_neighbors`, `reachable` (BFS), `shortest_path`, `collect_reachable`.
- Tables: `graph_node` (with FTS + HNSW indexes), `graph_edge`, `hyperedge_hub`, `source_of` (with decimal weight), `target_of` (with decimal weight), `petri_net`, `petri_place`, `petri_transition`, `pre_arc`, `post_arc`, `petri_marking`.

### Usage

```rust
use catgraph_surreal::{init_schema, init_schema_v2};
use catgraph_surreal::cospan_store::CospanStore;       // V1
use catgraph_surreal::node_store::NodeStore;            // V2
use catgraph_surreal::hyperedge_store::HyperedgeStore;  // V2

let db = Surreal::new::<Mem>(()).await?;
db.use_ns("test").use_db("test").await?;
init_schema(&db).await?;      // V1 tables
init_schema_v2(&db).await?;   // V2 tables (can coexist)

// V1: embedded array roundtrip
let v1 = CospanStore::new(&db);
let id = v1.save(&my_cospan).await?;
let loaded: Cospan<char> = v1.load(&id).await?;

// V2: graph-native decomposition
let v2 = HyperedgeStore::new(&db);
let hub_id = v2.decompose_cospan(&cospan, "reaction", props, |c| c.to_string()).await?;
let sources = v2.sources(&hub_id).await?;
let reconstructed: Cospan<char> = v2.reconstruct_cospan(&hub_id).await?;
```

### Dependencies

`catgraph`, `surrealdb` 3.0.5 (kv-mem), `surrealdb-types` 3.0.5, `serde` + `serde_json`, `tokio`, `thiserror`, `rust_decimal`, `petgraph`

### catgraph core dependencies

`petgraph`, `union-find`, `permutations`, `itertools`, `rayon`, `num`, `either`, `log`, `thiserror`, `rust_decimal`. Dev-only: `env_logger`, `proptest`, `criterion`.

## Testing

### Running Tests

```bash
cargo test --workspace        # Run all 640 tests (492 catgraph + 148 bridge), 1 ignored
cargo test                    # Run catgraph-only tests (492: 290 unit + 202 integration)
cargo test -p catgraph-surreal # Run bridge crate tests (148: 19 unit + 129 integration)
cargo test --examples         # Compile-check all 19 examples
cargo bench --no-run          # Compile-check all 4 benchmarks
cargo clippy                  # Lint checks
cargo tarpaulin --out Stdout  # Coverage report
```

### Test Patterns

Tests typically use simple types for Lambda:
- `char` for readable examples
- `()` for untyped tests
- Custom enums (e.g., `Color { Red, Green, Blue }`) for typed examples

## Common Patterns

### Creating Identity Morphisms

```rust
let id = Cospan::identity(&vec!['a', 'b', 'c']);
let named_id = NamedCospan::identity(&types, &prenames, |n| (n, n));
```

### Composition

```rust
let result = morphism1.compose(&morphism2)?;  // returns Result<_, CatgraphError>
```

### Monoidal Product

```rust
let mut combined = morphism1;
combined.monoidal(morphism2);
```

### Permutations

```rust
use permutations::Permutation;
let p = Permutation::rotation_left(3, 1);
let cospan = Cospan::from_permutation(p, &types, types_as_on_domain)?;
```

## Type Constraints

- `Lambda: Sized + Eq + Copy + Debug` (catgraph core); `Persistable: Sized + Eq + Clone + Debug` (persistence)
- Names often need `Eq + Clone` (and `Hash` for validation)
- Group elements need `One + MulAssign + Eq + Clone`

## Public API (hardened, tested)

| Module | What it provides |
|--------|-----------------|
| `category.rs` | Core traits: `HasIdentity`, `Composable`, `ComposableMutating` |
| `cospan.rs` | Pushout composition via union-find, identity fast-paths |
| `named_cospan.rs` | Port-labeled cospans for wiring-style composition |
| `span.rs` | Pullback composition (dual of cospan) |
| `span.rs` — `Rel` | Relation algebra: `new`/`new_unchecked`, `is_reflexive`, `is_symmetric`, `is_transitive`, `is_antisymmetric`, `subsumes`, `union`, `intersection`, `complement`, `is_equivalence_rel`, `is_partial_order` |
| `monoidal.rs` | Tensor product, symmetric braiding, `GenericMonoidalMorphism` |
| `frobenius/operations.rs` | String diagram morphisms, `two_layer_simplify` (4 rules), `from_permutation`, `from_decomposition` |
| `frobenius/morphism_system.rs` | DAG-based black box interpretation: name morphisms, compose by reference, topological resolution via `fill_black_boxes`. Uses `Contains` + `InterpretableMorphism` traits. |
| `e1_operad.rs` | Little intervals operad: containment, overlap, coalescence, monoid homomorphism. Fallible constructor with epsilon tolerance. |
| `e2_operad.rs` | Little disks operad: 2D containment, coalescence, `from_e1_config` embedding. Fallible constructor with epsilon tolerance. |
| `temperley_lieb.rs` | Brauer/Temperley-Lieb algebra generators (`e_i`, `s_i`), dagger, `simplify`, composition via `ExtendedPerfectMatching` |
| `wiring_diagram.rs` | Operadic substitution built on `NamedCospan` |
| `petri_net.rs` | `PetriNet`, `Transition`, `Marking`: construction, `enabled`, `fire`, `reachable`, `can_reach`, `from_cospan`, `transition_as_cospan`, `parallel`, `sequential` |
| `finset.rs` | `Permutation`, `OrderPresSurj`, `OrderPresInj`, `Decomposition`, epi-mono factorization |
| `linear_combination.rs` | Vector space over morphisms (ring axioms, parallel mul) |
| `interval.rs` | `DiscreteInterval` (composition, intersection, containment), `ParallelIntervals` (tensor, direct sum) |
| `complexity.rs` | `Complexity` trait, `StepCount` (sequential composition) |
| `computation_state.rs` | `ComputationState` (step, complexity, to_interval mapping) |
| `adjunction.rs` | `ZPrimeOps` trait, `AdjunctionVerification` (triangle identities), `AdjunctionIrreducibility` |
| `bifunctor.rs` | `TensorProduct` trait, `IntervalTransform`, verify_associativity/unit_laws/symmetry |
| `coherence.rs` | `CoherenceVerification`, `DifferentialCoherence`, verify_associator/unitor/braiding coherence |
| `stokes.rs` | `TemporalComplex` (simplicial complex), `ConservationResult`, `StokesError` |

## Parallelization

The library uses rayon for parallel computation with adaptive thresholds:

| Module | Parallelized Operation | Threshold |
|--------|------------------------|-----------|
| `linear_combination.rs` | `Mul` impl, `linear_combine` | 32 terms |
| `temperley_lieb.rs` | `non_crossing` checks | 8 elements |
| `named_cospan.rs` | `find_nodes_by_name_predicate` | 256 elements |
| `frobenius/operations.rs` | `hflip` block mutations | 64 blocks |

### Async Integration

All parallelism is rayon-based (CPU-bound). For tokio integration, use **tokio-rayon** (not `spawn_blocking`, which is for I/O blocking). Rayon's work-stealing thread pool is optimized for CPU parallelism.

```rust
use std::sync::LazyLock;
use rayon::ThreadPoolBuilder;
use tokio_rayon::AsyncThreadPool;

static EXEC: LazyLock<Executor> = LazyLock::new(|| Executor::new());

struct Executor { pool: rayon::ThreadPool }

impl Executor {
    fn new() -> Self {
        Self { pool: ThreadPoolBuilder::new().build().unwrap() }
    }
    async fn run<F, R>(&self, f: F) -> R
    where F: FnOnce() -> R + Send + 'static, R: Send + 'static {
        self.pool.spawn_async(f).await
    }
}

// Usage:
let result = EXEC.run(move || {
    cospan_a.compose(&cospan_b) // rayon work-stealing kicks in above thresholds
}).await?;
```

## Clippy Preferences

Rust 2024 edition. Common patterns:
- Use `matches!` macro instead of match expressions returning bool
- Use `.is_multiple_of()` instead of `% n == 0`
- Collapse nested `if let` with `&&` (let chains)

## Future Work

| Area | Notes |
|------|-------|
| Compact closure (Fong-Spivak §3.1) | Cup/cap morphisms, name bijection — schema ready (hub with source_count=0) |
| CospanAlgebra trait (Fong-Spivak §2.1) | Lax monoidal functor from cospans to sets |
| WeightedCospan | `weight: option<decimal>` on source_of/target_of already in schema |
| Magnitude enrichment | Requires WeightedCospan + Tsallis entropy computation |
| Benchmark tuning | Criterion benchmarks exist; rayon thresholds not yet validated with data |
| LayeredMorphism | ~76 LOC duplication between FrobeniusMorphism and GenericMonoidalMorphism. Generic extraction deferred (net negative: divergent trait bounds). |

## API Scope

catgraph implements **applied category theory for compositional systems** — specifically Fong-Spivak-style string diagrams and cospans (source/target hypergraph semantics). It is NOT a general category theory library.
