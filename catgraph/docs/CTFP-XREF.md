# Category Theory for Programmers × catgraph: Concept Cross-Reference

> **Book:** Bartosz Milewski, *Category Theory for Programmers* (2019)
> **Library:** catgraph v0.10.1 — 1080+ workspace tests, Rust 2024 edition
> **Updated:** 2026-04-09
> **Purpose:** Map CTFP's pedagogical progression to catgraph's applied implementation. Companion to `FONG-SPIVAK-XREF.md`.

---

## Scope and Orientation

CTFP teaches category theory through a programming lens (Haskell/C++), building from basic categories up through enriched categories and Kan extensions. catgraph is an *applied* category theory library implementing Fong-Spivak-style compositional systems — cospans, string diagrams, hypergraph rewriting, and lattice gauge theory.

The overlap is structural: catgraph implements many of the abstract concepts CTFP explains, but in a domain-specific way. Where CTFP shows how functors appear as Haskell type constructors, catgraph implements hypergraph functors that preserve Frobenius structure. Where CTFP discusses monoidal categories abstractly, catgraph provides concrete tensor products of cospans and string diagrams.

**Status legend:** ✅ Implemented — ⚠️ Partial — ❌ Not implemented — ➖ Out of scope

---

## Part I: The Basics

### Ch 1–3: Categories, Types, and Composition

| CTFP Concept | Status | catgraph Module | Notes |
|---|---|---|---|
| Category (objects + morphisms) | ✅ | `category.rs` | `HasIdentity`, `Composable`, `ComposableMutating` traits |
| Identity morphism | ✅ | `category.rs` | `HasIdentity::identity(on_this)` for cospans, spans, Frobenius morphisms |
| Composition (associative) | ✅ | `category.rs` | `Composable::compose` returns `Result<Self, CatgraphError>` |
| Domain and codomain | ✅ | `category.rs` | `Composable::domain()`, `codomain()` — typed boundary vectors |
| Composition size mismatch | ✅ | `errors.rs` | `CatgraphError::CompositionSizeMismatch { expected, actual }` |
| Types as objects | ✅ | `cospan.rs` | `Lambda` type parameter plays the role of type labels on objects |

**Key difference:** CTFP's category is **Hask** (types and functions). catgraph's primary category is **Cospan_Λ** (finite typed sets and cospans between them). Objects are `Vec<Lambda>` — lists of typed wires.

### Ch 4: Kleisli Categories

| CTFP Concept | Status | catgraph Module | Notes |
|---|---|---|---|
| Kleisli category | ➖ | — | catgraph doesn't model computational effects |
| Kleisli composition | ➖ | — | No monadic composition; cospans compose via pushout |

### Ch 5: Products and Coproducts

| CTFP Concept | Status | catgraph Module | Notes |
|---|---|---|---|
| Product (pullback) | ✅ | `span.rs` | `Span::compose` implements pullback via intersection of middle pairs |
| Coproduct (pushout) | ✅ | `cospan.rs` | `Cospan::compose` implements pushout via union-find, O(n·α(n)) |
| Initial object | ✅ | `cospan.rs` | Empty boundary (`Vec::new()`) — the 0-wire object |
| Terminal object | ✅ | `span.rs` | Empty boundary in spans |

**catgraph's pushout is the core composition mechanism.** Every cospan composition is literally computing a coproduct in FinSet. This is the operational heart of the library, benchmarked in `benches/pushout.rs` at sizes 4–1024.

### Ch 6: Simple Algebraic Data Types

| CTFP Concept | Status | catgraph Module | Notes |
|---|---|---|---|
| Sum types (coproducts) | ⚠️ | `cospan.rs` | Monoidal product (`Monoidal::monoidal`) is the coproduct of boundary sets |
| Product types | ⚠️ | `span.rs` | Span monoidal product |
| Semiring of types | ➖ | — | catgraph doesn't model type algebra directly |

---

## Part I (cont.): Functors and Natural Transformations

### Ch 7: Functors

| CTFP Concept | Status | catgraph Module | Notes |
|---|---|---|---|
| Functor (structure-preserving map) | ✅ | `hypergraph_functor.rs` | `HypergraphFunctor` trait — preserves composition, identity, and Frobenius structure (Fong-Spivak §2.3 Eq. 12) |
| Functor laws (identity, composition) | ✅ | `tests/hypergraph_functor.rs` | 21 integration tests verify functoriality |
| Relabeling functor | ✅ | `hypergraph_functor.rs` | `RelabelingFunctor`: Cospan\<L1\> → Cospan\<L2\> via label relabeling |
| Cospan-to-Frobenius functor | ✅ | `hypergraph_functor.rs` | `CospanToFrobeniusFunctor`: decomposes cospans into string diagrams via epi-mono factorization (Prop 3.8) |
| Endofunctor | ➖ | — | Not explicitly modeled (catgraph works across categories, not within Hask) |

**Key difference:** CTFP presents functors as Haskell's `fmap`. catgraph's functors are *hypergraph functors* — they must additionally preserve the Frobenius algebra structure (merge, split, unit, counit) on every object. This is a much stronger condition than just preserving composition.

### Ch 8: Functoriality

| CTFP Concept | Status | catgraph Module | Notes |
|---|---|---|---|
| Bifunctor | ✅ | `bifunctor.rs` | `TensorProduct` trait, `IntervalTransform`, `tensor_bimap`/`tensor_first`/`tensor_second` |
| Bifunctor laws | ✅ | `tests/bifunctor_laws.rs` | 6 tests: associativity, unit laws, symmetry |
| Profunctor | ➖ | — | Not implemented |

### Ch 10: Natural Transformations

| CTFP Concept | Status | catgraph Module | Notes |
|---|---|---|---|
| Natural transformation | ⚠️ | `cospan_algebra.rs` | Morphisms between cospan-algebras include a monoidal natural transformation component (Fong-Spivak Def 2.2) |
| Naturality condition | ⚠️ | `tests/cospan_algebra.rs` | Verified via functoriality coherence tests |

**Key difference:** CTFP shows natural transformations as polymorphic functions (`alpha :: F a -> G a`). catgraph doesn't have a standalone `NaturalTransformation` trait. Instead, naturality appears implicitly in the coherence conditions verified for cospan-algebra morphisms and hypergraph functors.

---

## Part II: Monoidal and Higher Structure

### Ch 18: Adjunctions

| CTFP Concept | Status | catgraph Module | Notes |
|---|---|---|---|
| Adjunction (L ⊣ R) | ✅ | `adjunction.rs` | `AdjunctionVerification` with triangle identity checks |
| Unit (η) and counit (ε) | ✅ | `adjunction.rs` | Part of `AdjunctionVerification` |
| Triangle identities | ✅ | `tests/adjunction_laws.rs` | 5 tests including gap measurement |
| Free–forgetful adjunction (Cospan_– ⊣ Ob) | ❌ | — | Fong-Spivak Thm 3.14; not yet exposed as API |

**catgraph connection:** The adjunction `Cospan_– ⊣ Ob` (CTFP Ch 18 general theory, Fong-Spivak Thm 3.14 specific instance) asserts that Cospan_Λ is the *free* hypergraph category on Λ. catgraph implements this implicitly — it works inside Cospan_Λ directly — but doesn't expose the universal property as an API.

### Ch 22: Monads Categorically / Ch 22.1: Monoidal Categories

| CTFP Concept | Status | catgraph Module | Notes |
|---|---|---|---|
| Monoidal category | ✅ | `monoidal.rs` | `Monoidal` trait (tensor product), `SymmetricMonoidalMorphism` (braiding + permutations) |
| Tensor product (⊗) | ✅ | `monoidal.rs` | `Monoidal::monoidal(&mut self, other)` — disjoint union of wire sets |
| Unit object (I) | ✅ | `cospan.rs` | Empty boundary: `Cospan::identity(&vec![])` |
| Symmetric braiding (σ) | ✅ | `monoidal.rs` | `SymmetricMonoidalMorphism::from_permutation` + `permute_side` |
| Associator coherence | ✅ | `coherence.rs` | `CoherenceVerification::verify_associator` |
| Unitor coherence | ✅ | `coherence.rs` | `verify_left_unitor`, `verify_right_unitor` |
| Braiding coherence | ✅ | `coherence.rs` | `verify_braiding` |
| Pentagon + triangle diagrams | ✅ | `tests/coherence_laws.rs` | 7 tests covering all 4 coherence axioms |
| Monad (T, μ, η) | ➖ | — | catgraph doesn't model monads in the Haskell sense |

**Key difference:** CTFP's monoidal categories are abstract. catgraph's monoidal structure is *strict* (associator and unitors are identities, not just isomorphisms). This is a legitimate simplification because Cospan_Λ is naturally strict, and Mac Lane's coherence theorem (CTFP Ch 22.1, Fong-Spivak Thm 3.22) guarantees this loses no generality.

### Ch 23: Comonads

| CTFP Concept | Status | catgraph Module | Notes |
|---|---|---|---|
| Comonoid (δ, ε) | ✅ | `frobenius/operations.rs` | `FrobeniusOperation::Comultiplication(λ)` (δ) and `Counit(λ)` (ε) |

catgraph's Frobenius structure bundles a monoid *and* a comonoid on each object, satisfying the Frobenius compatibility law. The comonoid half (comultiply + counit) is exactly what CTFP's comonads look like when specialized to the discrete/set-theoretic case.

---

## Part II (cont.): Limits, Colimits, and Universal Constructions

### Ch 12: Limits and Colimits

| CTFP Concept | Status | catgraph Module | Notes |
|---|---|---|---|
| Limit (general) | ⚠️ | `span.rs` | Span composition computes pullbacks (a specific limit) |
| Colimit (general) | ⚠️ | `cospan.rs` | Cospan composition computes pushouts (a specific colimit) |
| Pullback | ✅ | `span.rs` | `Span::compose` — intersection-based, benchmarked in `benches/pullback.rs` |
| Pushout | ✅ | `cospan.rs` | `Cospan::compose` — union-find, O(n·α(n)), benchmarked in `benches/pushout.rs` |
| Equalizer / Coequalizer | ❌ | — | Not implemented as standalone constructions |

**catgraph builds on colimits.** The entire cospan composition machinery is pushout computation. This is where CTFP's abstract notion of colimit becomes the concrete operational core of catgraph.

### Ch 15–16: Yoneda Lemma / Yoneda Embedding

| CTFP Concept | Status | catgraph Module | Notes |
|---|---|---|---|
| Yoneda lemma | ❌ | — | Not directly implemented |
| Representable functors | ❌ | — | Not implemented |
| Yoneda embedding | ➖ | — | Would require modeling presheaf categories |

The Yoneda perspective is implicitly present in catgraph's name bijection (compact_closed.rs): the isomorphism H(X, Y) ≅ H(I, X⊗Y) is a specific instance of Yoneda-like reasoning applied to self-dual compact closed categories.

---

## Part III: Advanced Topics

### Ch 25: Algebras for Monads / F-Algebras

| CTFP Concept | Status | catgraph Module | Notes |
|---|---|---|---|
| F-algebra | ⚠️ | `frobenius/morphism_system.rs` | `MorphismSystem` resolves DAGs of composable morphisms — conceptually an algebra over the free monad of string diagram combinators |
| Catamorphism / fold | ➖ | — | Not directly exposed |
| Initial algebra | ➖ | — | Not implemented |

### Ch 26: Algebras for Monads

| CTFP Concept | Status | catgraph Module | Notes |
|---|---|---|---|
| Algebra over a monad | ⚠️ | `cospan_algebra.rs` | `CospanAlgebra` trait — lax monoidal functor Cospan_Λ → Set. This is the Fong-Spivak notion of "algebra" for the cospan monad |
| Eilenberg-Moore category | ❌ | — | Not implemented |
| Kleisli category | ➖ | — | Not relevant to catgraph's domain |

### Ch 27: Kan Extensions

| CTFP Concept | Status | catgraph Module | Notes |
|---|---|---|---|
| Right Kan extension | ❌ | — | Not implemented |
| Left Kan extension | ❌ | — | Not implemented |
| Lan/Ran as adjunctions | ❌ | — | Not implemented |

Kan extensions would become relevant if catgraph needed to compute "best approximations" of functors along other functors — for instance, extending a partially-defined hypergraph functor to a full one. Currently not in scope.

### Ch 28: Enriched Categories

| CTFP Concept | Status | catgraph Module | Notes |
|---|---|---|---|
| Enriched category (C over V) | ⚠️ | roadmap | Core design thread for magnitude enrichment (Bradley-Vigneaux). `WeightedCospan` planned |
| Monoidal category as enrichment base | ✅ | `monoidal.rs` | Monoidal structure already implemented; serves as foundation for enrichment |
| Metric spaces as enriched categories | ❌ | — | Lawvere metric spaces not implemented, but relevant to magnitude/distance features |
| Enriched functor | ❌ | — | Planned as part of magnitude enrichment |
| Preorders as enriched categories | ⚠️ | `span.rs` | `Rel::is_partial_order()`, `is_equivalence_rel()` — relations checked but not modeled as enriched categories |

**This is the critical frontier.** CTFP Ch 28's enriched categories (especially §28.5 metric spaces) map directly to catgraph's planned magnitude enrichment. The idea: enrich hypergraph edges over [0,∞] with distance = −log(probability), making magnitude a scalar measure of effective complexity. See `docs/ROADMAP.md` and the Bradley-Vigneaux design thread.

---

## Frobenius Structure: catgraph's Core (Beyond CTFP)

CTFP mentions Frobenius algebras only in passing. catgraph makes them central. This table maps CTFP's scattered references to catgraph's unified Frobenius implementation.

| CTFP Reference | catgraph Implementation | Module |
|---|---|---|
| Monoid (Ch 22.1: μ, η) | `FrobeniusOperation::Multiplication(λ)`, `Unit(λ)` | `frobenius/operations.rs` |
| Comonoid (Ch 23) | `FrobeniusOperation::Comultiplication(λ)`, `Counit(λ)` | `frobenius/operations.rs` |
| Monoidal category (Ch 22.1) | `Monoidal`, `SymmetricMonoidalMorphism` | `monoidal.rs` |
| Symmetric braiding | `FrobeniusOperation::SymmetricBraiding(λ₁, λ₂)` | `frobenius/operations.rs` |
| Compact closed (implicit in adjunctions) | Cup/cap, zigzag identities, name bijection | `compact_closed.rs` |
| String diagrams (Ch 22 figures) | `FrobeniusMorphism<Lambda, Label>` — layered string diagrams | `frobenius/operations.rs` |

catgraph's `FrobeniusMorphism` is a first-class string diagram data structure. Each morphism is a sequence of layers (parallel blocks of operations). The `two_layer_simplify` function applies 4 rewrite rules (unit/counit cancellation, special law, identity elimination) to simplify diagrams — this is the computational engine behind Fong-Spivak's coherence theorem.

---

## Features in catgraph With No CTFP Counterpart

These are applied category theory features that go well beyond CTFP's scope:

| Feature | Module | Theoretical Basis | Why No CTFP Coverage |
|---|---|---|---|
| DPO hypergraph rewriting | `hypergraph/rewrite_rule.rs` | Ehrig et al. | Graph transformation theory, not covered in CTFP |
| Multiway evolution + BFS | `multiway/evolution_graph.rs` | Gorard, Wolfram | Non-deterministic computation / physics |
| Branchial foliation | `multiway/branchial.rs` | Gorard | Multiway systems theory |
| Ollivier-Ricci curvature | `multiway/ollivier_ricci.rs` | Ollivier 2009 | Discrete differential geometry |
| Wasserstein optimal transport | `multiway/wasserstein.rs` | Transportation theory | Measure theory, not covered in CTFP |
| Lattice gauge theory | `hypergraph/gauge.rs` | Wilson, Gorard | Physics application |
| Petri nets | `petri_net.rs` | Compositional Petri nets | Concurrency theory |
| Temperley-Lieb / Brauer algebra | `temperley_lieb.rs` | Knot theory, representation theory | Algebraic, not in CTFP's scope |
| E₁/E₂ operads | `e1_operad.rs`, `e2_operad.rs` | Little cubes (May) | Higher algebra, beyond CTFP |
| Wiring diagram operad | `wiring_diagram.rs` | Spivak | Operadic composition |
| Relation algebra | `span.rs` (`Rel`) | Tarski | Relations ∪ ∩ complement, reflexivity, transitivity |
| Linear combinations over morphisms | `linear_combination.rs` | Categorification | Free vector spaces on morphism sets |
| FinSet decomposition | `finset.rs` | Epi-mono factorization | Constructive set theory |
| Discrete interval / complexity | `interval.rs`, `complexity.rs` | Computation theory | Domain-specific |
| Temporal complex / Stokes | `stokes.rs` | Discrete exterior calculus | Physics application |
| SurrealDB persistence (V1 + V2) | `catgraph-surreal/` | — | Engineering, not theory |

---

## CTFP Topics Not in catgraph's Scope

These CTFP chapters cover concepts that catgraph intentionally does not implement, either because they belong to a different domain (programming language semantics) or because they are not needed for applied compositional systems.

| CTFP Chapter | Topic | Why Not in Scope |
|---|---|---|
| Ch 4, 20–21 | Kleisli categories, monads in programming | catgraph is about compositional systems, not computational effects |
| Ch 7.1 | Functors as type constructors (Maybe, List) | catgraph's functors are between mathematical categories, not Haskell types |
| Ch 9 | Function types / exponential objects | catgraph works in FinSet where exponentials are not central |
| Ch 11 | Declarative programming | Language paradigms, not mathematical structure |
| Ch 13 | Free monoids | catgraph uses free *hypergraph* categories instead (Fong-Spivak Thm 3.14) |
| Ch 14, 19 | Representable functors, free/forgetful | Abstract nonsense that catgraph satisfies implicitly via Cospan_Λ |
| Ch 15–16 | Yoneda lemma/embedding | Would need presheaf categories; not in current scope |
| Ch 24 | Topoi | catgraph's categories are not topoi |
| Ch 29 | Topoi (cont.) | Same |

---

## Reading Paths

### For understanding catgraph's mathematical foundations via CTFP:

1. **Ch 1–3** (Categories, composition) → `category.rs`
2. **Ch 5** (Products/coproducts) → `cospan.rs` (pushout), `span.rs` (pullback)
3. **Ch 7** (Functors) → `hypergraph_functor.rs`
4. **Ch 22.1** (Monoidal categories) → `monoidal.rs`, `coherence.rs`
5. **Ch 18** (Adjunctions) → `adjunction.rs`, then Fong-Spivak §3.2 for the free-forgetful adjunction
6. **Ch 28** (Enriched categories) → `docs/ROADMAP.md` for the magnitude enrichment plan

### For understanding catgraph features not in CTFP:

Start with `docs/FONG-SPIVAK-XREF.md` — it covers the string diagram / cospan algebra material that CTFP omits entirely. Then see the Frobenius, hypergraph rewriting, and multiway modules.

---

## Summary Statistics

| Metric | CTFP (31 chapters) | catgraph |
|---|---|---|
| Core categorical concepts | ~45 | ~22 implemented |
| Chapters with direct catgraph mapping | 14 / 31 | — |
| Chapters partially relevant | 6 / 31 | — |
| Chapters out of scope | 11 / 31 | — |
| catgraph features beyond CTFP | — | 17 modules |
| Tests covering CTFP-mapped concepts | — | 500+ |
| Tests covering beyond-CTFP features | — | 500+ |

The overlap between CTFP and catgraph is approximately 45% by chapter coverage, but the *depth* of overlap in the relevant areas (composition, monoidal structure, adjunctions, enriched categories) is high. catgraph implements the applied side of what CTFP teaches abstractly, and extends far beyond CTFP into hypergraph rewriting, gauge theory, and multiway systems that have no CTFP counterpart.
