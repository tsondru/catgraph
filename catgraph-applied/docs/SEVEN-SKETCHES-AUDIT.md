# Seven Sketches Coverage Audit (catgraph-applied v0.1.0)

> **Paper:** Fong & Spivak, *Seven Sketches in Compositionality: An Invitation to Applied Category Theory* (arXiv:1803.05316v3, 12 Oct 2018)
> **Library:** catgraph-applied v0.1.0 (workspace member of catgraph v0.11.0)
> **Date:** 2026-04-16 (Phase 5 initial audit)
> **Method:** read all 334 pages of the textbook, cross-checked each numbered definition/theorem/example against catgraph-applied source and catgraph core
>
> **Note on scope:** *Seven Sketches* is a 334-page textbook covering seven topics in applied CT. Only **Chapters 4, 5, and 6** contain formal content relevant to catgraph-applied's modules. Chapters 1–3 (orders, enrichment, databases) and Chapter 7 (toposes) establish foundational CT that catgraph core already provides or that is outside catgraph's scope entirely.
>
> **Relationship to catgraph core audit:** The core catgraph crate tracks Fong & Spivak's *Hypergraph Categories* (arXiv:1806.08304v3, 2019) — the research paper that formalizes the §6.3 content into a full equivalence theorem. See [`catgraph/docs/FONG-SPIVAK-AUDIT.md`](../../catgraph/docs/FONG-SPIVAK-AUDIT.md) for the core audit. This audit covers the *textbook* content that goes beyond that paper: decorated cospans, operads and their algebras, props, signal flow, and wiring diagrams for monoidal/compact-closed/hypergraph categories.

**Status legend:**
- ✅ DONE — implemented and tested
- ⚠️ PARTIAL — implementation exists but is incomplete or doesn't fully exhibit the paper's structure
- ❌ MISSING — not implemented in catgraph-applied (or catgraph core)
- ➖ N/A — theoretical / motivational / pedagogical, no implementation expected
- 🔗 IN CORE — implemented in catgraph core (not catgraph-applied); noted for completeness

## Summary

| Chapter/Section | DONE | PARTIAL | MISSING | N/A | IN CORE | Total |
|---|---|---|---|---|---|---|
| §4.4 Categorification + monoidal cats | 1 | 0 | 0 | 3 | 2 | 6 |
| §4.5 Compact closed categories | 0 | 0 | 0 | 2 | 3 | 5 |
| §5.2 Props and presentations | 0 | 1 | 3 | 3 | 0 | 7 |
| §5.3 Signal flow graphs | 0 | 0 | 4 | 2 | 0 | 6 |
| §5.4 Graphical linear algebra | 0 | 0 | 2 | 1 | 0 | 3 |
| §6.2 Colimits and connection | 0 | 0 | 0 | 2 | 4 | 6 |
| §6.3 Hypergraph categories | 1 | 1 | 0 | 2 | 5 | 9 |
| §6.4 Decorated cospans | 1 | 2 | 2 | 1 | 0 | 6 |
| §6.5 Operads and their algebras | 3 | 2 | 2 | 1 | 0 | 8 |
| **TOTAL** | **6** | **6** | **13** | **17** | **14** | **56** |

**Headline numbers:**
- **11% DONE / 11% PARTIAL / 23% MISSING / 30% N/A / 25% IN CORE**
- Of the 56 audited items, 14 are already in catgraph core (the research paper's content), 17 are N/A (pedagogical), leaving **25 implementable items** of which **6 are DONE, 6 PARTIAL, 13 MISSING**.
- Of implementable items: **24% DONE / 24% PARTIAL / 52% MISSING**
- The 13 missing items cluster in: props/presentations (§5.2–5.4, 9 items) and decorated cospans / operad algebras (§6.4–6.5, 4 items).

---

## Per-section detail

### §4.4 Categorification (pp. 132–139)

| Item | Status | Location | Notes |
|---|---|---|---|
| Rough Def 4.45: symmetric monoidal category | 🔗 | catgraph::monoidal | `Monoidal` + `SymmetricMonoidalMorphism` traits |
| Remark 4.46: strict SMC (Mac Lane coherence) | 🔗 | catgraph core design | catgraph works in the strict case |
| Remark 4.47: non-rough definition reference | ➖ | — | theoretical pointer |
| Ex 4.49: (Set, {1}, ×) monoidal structure | ➖ | — | motivational example |
| Ex 4.50: wiring diagram for monoidal composition | ✅ | catgraph-applied::wiring_diagram | `WiringDiagram` implements `Composable` + `Monoidal` for exactly this diagram interpretation |
| Rough Def 4.51: V-category (enriched in SMC) | ➖ | — | theoretical; catgraph uses Set-enrichment |

### §4.5 Profunctors form a compact closed category (pp. 139–146)

| Item | Status | Location | Notes |
|---|---|---|---|
| Def 4.58: dual, unit η, counit ε, snake equations | 🔗 | catgraph::compact_closed | cup/cap functions, zigzag tests |
| Prop 4.60: compact closed ⟹ monoidal closed | 🔗 | catgraph core (implicit) | catgraph relies on this via Prop 6.66 |
| Ex 4.61: Corel as compact closed category | 🔗 | catgraph::span::Rel | `Rel` exists; corelation structure implicit |
| Thm 4.63: Prof_V is compact closed | ➖ | — | theoretical; profunctor categories not implemented |
| Ex 4.66: snake equations for Prof_V | ➖ | — | theoretical verification |

### §5.2 Props and presentations (pp. 149–158)

| Item | Status | Location | Notes |
|---|---|---|---|
| Def 5.2: prop (symmetric strict monoidal category, Ob = ℕ) | ❌ | — | no explicit `Prop` type; catgraph uses general SMC traits |
| Def 5.11: prop functor | ➖ | — | definition only |
| Def 5.13: (m,n)-port graph | ⚠️ | catgraph-applied::petri_net | `PetriNet` is a bipartite graph with typed ports; not literally a port graph but structurally adjacent. `WiringDiagram` inner/outer circles are closer. |
| Def 5.25: free prop on a signature Free(G) | ❌ | — | no free prop construction; catgraph uses Frobenius generators directly |
| Def 5.30: G-generated prop expressions | ❌ | — | no syntactic prop expression type |
| Rough Def 5.33: presentation (G, s, t, E) for a prop | ❌ | — | no presentation type |
| Remark 5.34: universal property of presentations | ➖ | — | theoretical |
| Prop 5.29: universal property of Free(G) | ➖ | — | theoretical |

### §5.3 Simplified signal flow graphs (pp. 159–168)

| Item | Status | Location | Notes |
|---|---|---|---|
| Def 5.36: rig (semiring) | ➖ | — | catgraph uses `num` traits (Ring, Zero, One) but doesn't define a standalone rig type |
| Def 5.45: SFG_R = Free(G_R) (signal flow graphs as free prop) | ❌ | — | no signal flow graph implementation |
| Def 5.50: Mat(R) prop of R-matrices | ❌ | — | no matrix-prop type |
| Thm 5.53: prop functor S: SFG_R → Mat(R) | ❌ | — | functorial semantics not implemented |
| Prop 5.54: matrix S(g) describes input→output amplification | ➖ | — | theoretical consequence |
| Eq 5.52: generator → matrix table (copy, discard, add, zero, scalar) | ❌ | — | Frobenius generators exist in catgraph core but no matrix interpretation functor |

### §5.4 Graphical linear algebra (pp. 168–178)

| Item | Status | Location | Notes |
|---|---|---|---|
| Thm 5.60: presentation of Mat(R) from Frobenius + rig equations | ❌ | — | no presentation-based Mat(R) |
| Def 5.65: monoid object in SMC (commutative monoid axioms) | ❌ | — | catgraph has `FrobeniusOperation` (monoid + comonoid) but no standalone `MonoidObject` in general SMC |
| Thm 5.87: hypergraph category from linear relations | ➖ | — | LinRel deferred (same as core audit) |

### §6.2 Colimits and connection (pp. 184–196)

| Item | Status | Location | Notes |
|---|---|---|---|
| Def 6.1: initial object | ➖ | — | pedagogical; catgraph uses ∅ as monoidal unit |
| Def 6.11: coproduct | ➖ | — | pedagogical; catgraph monoidal product is coproduct on FinSet |
| Def 6.19: pushout | 🔗 | catgraph::cospan | union-find pushout composition |
| Prop 6.32: finite colimits ⟺ initial + pushouts | 🔗 | catgraph core (implicit) | FinSet has both |
| Thm 6.37: colimit formula as equivalence classes | 🔗 | catgraph::cospan | pushout via union-find is exactly this formula |
| Def 6.43 + 6.45: cospan, Cospan_C category | 🔗 | catgraph::cospan | `Cospan<Lambda>` with pushout composition |

### §6.3 Hypergraph categories (pp. 197–203)

| Item | Status | Location | Notes |
|---|---|---|---|
| Def 6.52: Frobenius structure (μ, η, δ, ε + 9 axioms) | 🔗 | catgraph::frobenius | `FrobeniusOperation`, 8 axiom tests |
| Def 6.54: spider s_{m,n} | 🔗 | catgraph::frobenius | `from_decomposition` constructs spiders from generators |
| Thm 6.55: spider theorem (connected diagrams = spiders) | ⚠️ | catgraph::frobenius | `two_layer_simplify` reduces connected diagrams; full spider-theorem statement not an explicit test, but the simplification achieves it |
| Thm 6.58: free prop on Frobenius ≅ Cospan_FinSet | 🔗 | catgraph::cospan_algebra + hypergraph_functor | `CospanToFrobeniusFunctor` (Prop 3.8 in the research paper) |
| Def 6.60: hypergraph category | 🔗 | catgraph::hypergraph_category | `HypergraphCategory` trait |
| Ex 6.61: Cospan_C is a hypergraph category | 🔗 | catgraph::hypergraph_category | `impl HypergraphCategory for Cospan<Lambda>` |
| Ex 6.64: Corel is a hypergraph category | ❌ | — | `Rel` exists in catgraph::span but `HypergraphCategory` impl for Corel is missing. **Same status as in core audit (Ex 2.15).** |
| Prop 6.66: hypergraph cats are self-dual compact closed | 🔗 | catgraph::compact_closed | cup/cap from η;δ and μ;ε |
| Temperley-Lieb as diagrammatic SMC (spider-theorem adjacent) | ✅ | catgraph-applied::temperley_lieb | `BrauerMorphism` composition via connected components + loop counting; TL generators e_i; Frobenius-law-adjacent relations tested (e_i² = δ·e_i, Jones relations) |

### §6.4 Decorated cospans (pp. 203–211)

| Item | Status | Location | Notes |
|---|---|---|---|
| Rough Def 6.68: symmetric monoidal functor (F, φ) | ➖ | — | theoretical; catgraph uses `HypergraphFunctor` |
| Def 6.75: F-decorated cospan | ⚠️ | catgraph-applied::petri_net | `PetriNet::from_cospan` / `transition_as_cospan` bridges between Petri net structure and cospans, but there is no general `DecoratedCospan<F>` type. The Petri net is a *specific* decorated cospan where the decoration functor sends N to the set of transition-weighted bipartite graphs on N. |
| Thm 6.77: Cospan_F is a hypergraph category | ⚠️ | catgraph-applied::petri_net | `PetriNet::parallel` (monoidal product) and `PetriNet::sequential` (composition) implement the Cospan_F operations for the Petri net decoration functor, but don't prove the general theorem. No `HypergraphCategory` impl on `PetriNet`. |
| Ex 6.79–6.86: Circ functor, decorated cospan composition for circuits | ❌ | — | no circuit decoration functor; the textbook's running example |
| Ex 6.88: closed circuits via η;x;ε composition | ❌ | — | no closed-circuit construction |
| Petri net cospan bridge (pre/post arc weights as left/right legs) | ✅ | catgraph-applied::petri_net | `from_cospan`, `transition_as_cospan` — multiplicity-weighted cospan bridge. `fire`, `enabled`, `reachable` for state-space exploration. |

### §6.5 Operads and their algebras (pp. 211–218)

| Item | Status | Location | Notes |
|---|---|---|---|
| Rough Def 6.91: operad (types, operations, substitution ∘_i, identities) | ✅ | catgraph::operadic + catgraph-applied::e1_operad, e2_operad | `Operadic` trait in core defines substitution with identity/associativity laws. E₁ and E₂ implement concrete operads with validated substitution. |
| Ex 6.93: Set operad (functions as operations) | ➖ | — | motivational example |
| Ex 6.94: Cospan operad (cospans as operations, substitution by pushout) | ✅ | catgraph-applied::wiring_diagram | `WiringDiagram` implements `Operadic` with cospan-pushout substitution. This IS the Cospan operad specialized to named cospans with inner/outer circles. |
| Eq 6.95: wiring diagram as cospan operation | ✅ | catgraph-applied::wiring_diagram | the `Operadic::substitute` implementation literally performs this: replace an inner circle with a sub-diagram, connecting ports by name |
| Def 6.97: operad O_C underlying any SMC C | ⚠️ | catgraph::operadic (trait) | the `Operadic` trait captures the abstract interface, but there is no generic construction that takes an arbitrary SMC and produces its underlying operad |
| Rough Def 6.98: operad functor | ❌ | — | no operad functor type; `HypergraphFunctor` covers the categorical (non-operadic) case |
| Def 6.99: operad algebra (F: O → Set) | ❌ | — | no operad algebra type; the textbook's Circ: Cospan → Set example (Ex 6.100) is not implemented |
| Prop 6.101: Cospan-algebras ≅ hypergraph props | ⚠️ | catgraph::cospan_algebra + equivalence | the per-Λ version (Thm 4.13 in the research paper) is verified in catgraph core. The operadic reformulation (Cospan-*algebras* in the operad sense ≅ hypergraph *props*) is not explicitly tested as such. |

---

## Critical findings

### What catgraph-applied implements well

1. **Operadic substitution (§6.5)** — `WiringDiagram` faithfully implements the Cospan operad (Ex 6.94) with name-matched port substitution. This is the textbook's primary concrete operad example. E₁ and E₂ operads demonstrate the abstract definition (Rough Def 6.91) with geometric substitution (affine rescaling).

2. **Temperley-Lieb / Brauer algebra (§6.3 adjacent)** — `BrauerMorphism` implements the diagrammatic category of perfect matchings with composition via connected components and closed-loop counting. TL generators and Jones relations are tested. This goes beyond the textbook (which mentions Frobenius diagrams and spiders but not TL specifically) into representation-theoretic territory.

3. **Petri net cospan bridge (§6.4 specialized)** — `PetriNet` implements a specific decorated cospan with multiplicity-weighted arc structure, BFS reachability, and parallel/sequential composition. The cospan bridge (`from_cospan` / `transition_as_cospan`) is well-tested.

4. **Linear combinations (§5.3 infrastructure)** — `LinearCombination<Coeffs, Target>` provides the free R-module over a basis set, with convolution product, functorial pushforward, and scalar operations. This is the algebraic infrastructure that the textbook presupposes (rigs, rings) but doesn't package as a standalone construct.

### Major gaps

1. **Props and presentations (§5.2)** — catgraph has no `Prop` type, no free prop construction `Free(G)`, no prop expressions, and no presentation machinery. This is the textbook's primary formalism for describing compositional theories. **Impact:** catgraph can compose morphisms in specific SMCs (Cospan, Frobenius, Brauer) but cannot express or reason about *theories* of composition abstractly.

2. **Signal flow graphs and Mat(R) (§5.3–5.4)** — No signal flow graph type, no matrix prop, no functorial semantics S: SFG_R → Mat(R). **Impact:** catgraph cannot demonstrate the textbook's main Ch 5 result — that signal flow diagrams have functorial matrix semantics. This is a significant missing application domain (linear systems, control theory).

3. **General decorated cospans (§6.4)** — `PetriNet` is a specific decorated cospan, but there is no general `DecoratedCospan<F>` parametric over a decoration functor. **Impact:** users cannot construct new hypergraph categories from symmetric monoidal functors F: (FinSet, +) → (Set, ×) without reimplementing the decorated cospan machinery.

4. **Operad algebras (§6.5 Def 6.99)** — No `OperadAlgebra` type (functor O → Set). The textbook shows that operad algebras unify decorated cospans and hypergraph props (Prop 6.101). **Impact:** the operadic perspective on compositional theories is missing.

### Items intentionally deferred

- **Ch 1–3** (orders, enrichment, databases): foundational CT already in catgraph core or out of scope
- **Ch 7** (toposes, sheaves, logic): out of scope for catgraph
- **LinRel examples** (Ex 6.65, Thm 5.87): deferred per core audit decision
- **Profunctor categories** (Thm 4.63): enriched profunctors are out of catgraph's scope

### Items that are implicit / "morally present" but not explicit

1. **Thm 6.55 (spider theorem)** — `two_layer_simplify` achieves spider reduction, but there is no explicit test asserting "any connected Frobenius diagram on m inputs and n outputs equals s_{m,n}".

2. **Def 6.97 (operad underlying an SMC)** — the `Operadic` trait captures the interface but the generic *construction* that derives an operad from any SMC is not automated.

3. **Prop 6.101 (Cospan-algebras ≅ hypergraph props)** — the per-Λ equivalence (Thm 4.13 in the research paper) is verified in catgraph core. Restating it in operadic language would be a test-only task.

---

## Inheritance from catgraph core

catgraph-applied builds on catgraph's F&S 2019 primitives. The following textbook items are **already implemented in catgraph core** and available to catgraph-applied modules:

| Textbook item | catgraph core location | catgraph-applied usage |
|---|---|---|
| Def 6.19: pushout composition | `cospan.rs` (union-find) | `WiringDiagram::substitute`, `PetriNet::from_cospan` |
| Def 6.43 + 6.45: Cospan_C | `cospan.rs`, `named_cospan.rs` | `WiringDiagram` wraps `NamedCospan` |
| Def 6.52: Frobenius structure | `frobenius/operations.rs` | `BrauerMorphism` TL generators; `WiringDiagram` operadic structure |
| Def 6.60: hypergraph category | `hypergraph_category.rs` | available for `PetriNet` to impl (currently missing) |
| Prop 6.66: self-dual compact closed | `compact_closed.rs` | `BrauerMorphism::dagger` uses compact closed structure |
| Thm 6.58: Cospan ≅ free Frobenius | `cospan_algebra.rs` | foundation for operadic substitution |
| Rough Def 4.45: SMC | `monoidal.rs` | `WiringDiagram`, `BrauerMorphism` implement `Monoidal` |

No duplication of F&S primitives in catgraph-applied — it depends on catgraph.

---

## Roadmap: priority gap closures for v0.2.0

### Tier 1 — high value, moderate effort

| Gap | Textbook ref | Effort | Notes |
|---|---|---|---|
| `DecoratedCospan<F>` generic type | Def 6.75, Thm 6.77 | 2–3 days | Parametric over decoration functor F: (C, +) → (Set, ×). Makes PetriNet a specialization. Enables circuit examples from §6.4.3. |
| `HypergraphCategory` impl for `PetriNet` | Def 6.60 via Thm 6.77 | 0.5 day | Given DecoratedCospan, the Frobenius structure comes from Cospan_C's colimits. |
| Spider theorem explicit test | Thm 6.55 | 0.5 day | Test that any connected Frobenius diagram simplifies to s_{m,n}. Already implicit in `two_layer_simplify`. |

### Tier 2 — medium value, enables new application domains

| Gap | Textbook ref | Effort | Notes |
|---|---|---|---|
| `Prop` type + `Free(G)` construction | Def 5.2, 5.25 | 1–2 days | Props are SMCs with Ob = ℕ. Free(G) from a signature (G, s, t). |
| `OperadAlgebra` type (F: O → Set) | Def 6.99 | 1 day | Functor from an operad to Set. Enables Circ example (Ex 6.100). |
| `OperadFunctor` type | Rough Def 6.98 | 0.5 day | Maps between operads preserving substitution + identities. |

### Tier 3 — lower priority, deferred unless needed

| Gap | Textbook ref | Effort | Notes |
|---|---|---|---|
| Signal flow graphs (SFG_R) | Def 5.45 | 2–3 days | Free prop on G_R generators. Requires Prop type from Tier 2. |
| Mat(R) prop + functorial semantics | Def 5.50, Thm 5.53 | 1–2 days | Matrix prop + S: SFG_R → Mat(R). Requires nalgebra or manual impl. |
| Presentation type (G, s, t, E) | Def 5.33 | 1 day | Equations on prop expressions. Requires Free(G) from Tier 2. |
| Graphical linear algebra (Thm 5.60) | §5.4.1 | 2 days | Presentation of Mat(R). Requires both Props and Mat(R). |
| Corel `HypergraphCategory` impl | Ex 6.64 | 0.5 day | catgraph::span::Rel exists; needs Frobenius structure on Corel |

---

## Recommendation for catgraph-applied v0.2.0 release notes

**catgraph-applied v0.1.0** implements the operadic substitution framework (§6.5 Rough Def 6.91 + Ex 6.94), the Temperley-Lieb/Brauer diagrammatic algebra (§6.3 spider-adjacent), Petri net decorated cospans (§6.4 specialized), little-intervals and little-disks operads (E₁, E₂), and formal linear combinations as algebraic infrastructure. It inherits all Fong-Spivak 2019 core from catgraph (cospans, Frobenius, hypergraph categories, compact closed, the §4 equivalence).

**For v0.2.0**, the priority is a generic `DecoratedCospan<F>` type (Def 6.75, Thm 6.77) that makes `PetriNet` a specialization and enables new application domains (circuits, chemical reaction networks). The props/presentations formalism (§5.2) and operad algebras (§6.5 Def 6.99) are secondary targets that would complete the textbook's Ch 5–6 theory stack.

---

## Cross-paper reconciliation: both F&S papers × all three workspace crates

This section maps every catgraph workspace module to its paper provenance (or lack thereof). Two papers are tracked:

- **[FS19]** = Fong & Spivak, *Hypergraph Categories* (arXiv:1806.08304v3, 2019) — tracked by [`catgraph/docs/FONG-SPIVAK-AUDIT.md`](../../catgraph/docs/FONG-SPIVAK-AUDIT.md)
- **[FS18]** = Fong & Spivak, *Seven Sketches in Compositionality* (arXiv:1803.05316v3, 2018) — tracked by this document

### catgraph core (v0.11.0) — all modules anchored to [FS19]

| Module | [FS19] ref | [FS18] ref | Notes |
|---|---|---|---|
| `cospan.rs` | §1 Eq 7, §2.1 | §6.2.5 Def 6.43–6.45 | pushout composition via union-find |
| `span.rs` / `Rel` | §2.3 Ex 2.15 | §5.2 Ex 5.8 (Rel prop) | pullback composition, relation algebra |
| `named_cospan.rs` | §1 Eq 4 | — | port-labeled cospans (catgraph extension) |
| `frobenius/` | §2.2 Def 2.5 | §6.3.1 Def 6.52 | Frobenius monoid generators + 9 axioms |
| `compact_closed.rs` | §3.1 Props 3.1–3.4 | §4.5.1 Def 4.58, Prop 6.66 | cup/cap, name/unname, compose_names |
| `cospan_algebra.rs` | §2.1 Def 2.2, §4.1 | — | PartitionAlgebra, NameAlgebra, functor lifting |
| `hypergraph_category.rs` | §2.3 Def 2.12 | §6.3.3 Def 6.60 | HypergraphCategory trait |
| `hypergraph_functor.rs` | §2.3 Eq 12, §3.2 Prop 3.8 | §6.3 Thm 6.58 | HypergraphFunctor, CospanToFrobeniusFunctor |
| `equivalence.rs` | §4 Thm 4.13 (= Thm 1.2) | — | CospanAlgebraMorphism, roundtrip |
| `monoidal.rs` | implicit | §4.4.3 Rough Def 4.45 | Monoidal, SymmetricMonoidalMorphism |
| `operadic.rs` (trait only) | §2.5 (motivational) | §6.5 Rough Def 6.91 | Operadic trait; concrete impls in catgraph-applied |
| `category.rs` | implicit | §3.2 Def 3.6 (pedagogical) | HasIdentity, Composable |
| `finset.rs` | §3.2 Lemma 3.6 | — | Permutation, Decomposition, epi-mono factorization |

### catgraph-applied (v0.1.0) — mixed provenance

| Module | [FS19] ref | [FS18] ref | Neither paper | Notes |
|---|---|---|---|---|
| `wiring_diagram.rs` | §2.5 Eq 6 (illustration) | §6.5 Ex 6.94 (Cospan operad), §4.4.2 wiring diagrams, §6.3.2 | — | Operadic substitution on named cospans. The *Operadic* trait is anchored to [FS18] §6.5; the wiring diagram interpretation is anchored to [FS18] §6.3.2 + §4.4.2. [FS19] only references wiring diagrams illustratively in §2.5. |
| `petri_net.rs` | — | §6.4 Def 6.75 (decorated cospan, specialized) | Baez-Pollard [BP17], Baez-Fong-Pollard [BFP16] | cospan bridge, fire/enable/reachable, parallel/sequential composition. The textbook cites [BFP16, BP17] as further reading for Petri nets as decorated cospans. The formal Petri-net-as-SMC treatment is from those papers, not from [FS18] or [FS19]. |
| `temperley_lieb.rs` | — | §6.3 (spider-adjacent) | Jones [Jon83], Kauffman [Kau87], Brauer [Bra37] | Brauer/TL algebra via perfect matchings, Jones relations, dagger. The textbook's Frobenius/spider material (§6.3) is the *context* for TL, but TL itself (non-crossing matchings, Jones polynomial, representation theory) is from the knot theory / representation theory literature, not from either F&S paper. |
| `linear_combination.rs` | — | §5.3.1 (rig infrastructure) | — | Free R-module R[T]. Provides the coefficient algebra that [FS18] §5.3 presupposes. Not a formal item in either paper — it's algebraic infrastructure. |
| `e1_operad.rs` | — | §6.5 Rough Def 6.91 | May [May72], Boardman-Vogt [BV73] | Little-intervals operad. [FS18] §6.5 defines operads abstractly; the *specific* E₁ operad is from the algebraic topology literature. |
| `e2_operad.rs` | — | §6.5 Rough Def 6.91 | May [May72], Boardman-Vogt [BV73] | Little-disks operad. Same: abstract operad definition from [FS18], specific E₂ construction from homotopy theory. |

### catgraph-physics (v0.2.0) — no F&S provenance

| Module | [FS19] ref | [FS18] ref | Actual provenance | Notes |
|---|---|---|---|---|
| `hypergraph/hypergraph.rs` | — | — | Wolfram [Wol20] | Typed hypergraph with source/target semantics |
| `hypergraph/rewrite_rule.rs` | — | — | Gorard [Gor20], Ehrig [EPS73] (DPO) | Double-pushout rewriting on hypergraphs |
| `hypergraph/evolution.rs` | — | — | Wolfram [Wol20], Gorard [Gor20] | Hypergraph evolution, BFS, causal invariance |
| `hypergraph/gauge.rs` | — | — | Gorard [Gor21] | Lattice gauge theory on hypergraph substrates |
| `hypergraph/evolution_cospan.rs` | uses `Cospan<Λ>` | — | catgraph bridge design | Cospan chain from evolution steps |
| `hypergraph/rewrite_span.rs` | uses `Span<Λ>` | — | catgraph bridge design | Span representation of rewrite rules |
| `hypergraph/multiway_cospan.rs` | uses `Cospan<Λ>` | — | catgraph bridge design | Multiway cospans |
| `multiway/evolution_graph.rs` | — | — | Wolfram [Wol20] | MultiwayEvolutionGraph, confluence diamonds |
| `multiway/branchial.rs` | — | — | Wolfram [Wol20], Gorard [Gor20] | BranchialGraph (per-step cross-sections) |
| `multiway/curvature.rs` | — | — | Ollivier [Oll09] | Ollivier-Ricci curvature on graphs |
| `multiway/wasserstein.rs` | — | — | Villani [Vil03] | Wasserstein-1 optimal transport |
| `multiway/branchial_spectrum.rs` | — | — | spectral graph theory | Laplacian eigendecomposition (nalgebra) |
| `multiway/branchial_analysis.rs` | — | — | rustworkx-core algorithms | Coloring, k-core, articulation points |

**catgraph-physics uses catgraph core types** (`Composable`, `Cospan`, `Span`) as categorical bridges, but its mathematical content is entirely from the Wolfram model / discrete differential geometry literature — neither F&S paper.

### Features not in either paper

| catgraph-applied module | Feature | Paper provenance |
|---|---|---|
| `temperley_lieb.rs` | Brauer algebra (perfect matchings with crossings) | Brauer [1937], not F&S |
| `temperley_lieb.rs` | Jones relations (e_i² = δ·e_i, far commutativity, braid) | Jones [1983], not F&S |
| `temperley_lieb.rs` | `LinearCombination` over Brauer diagrams | representation theory, not F&S |
| `temperley_lieb.rs` | `dagger` (adjoint / vertical reflection) | dagger-category structure, not F&S |
| `temperley_lieb.rs` | `non_crossing` detection | TL-specific, not F&S |
| `e1_operad.rs` | `go_to_monoid` homomorphism | algebraic topology, not F&S |
| `e1_operad.rs` | `coalesce_boxes` (inverse substitution) | catgraph design |
| `e2_operad.rs` | `from_e1_config` (E₁ → E₂ embedding) | standard embedding, not F&S |
| `petri_net.rs` | BFS reachability analysis | Petri net theory [Murata89], not F&S |
| `petri_net.rs` | Weighted arcs (`Decimal`) | quantitative Petri nets, not F&S |
| `linear_combination.rs` | Convolution product `Mul<Self>` | ring theory infrastructure |
| `linear_combination.rs` | `linearly_extend` / `inj_linearly_extend` | functorial pushforward |
| `wiring_diagram.rs` | Directed ports (`Dir::In`, `Dir::Out`, `Dir::Undirected`) | catgraph design extension |

### Overlap between papers

The following items appear in both [FS18] and [FS19]. The core audit ([FS19]) is authoritative for these; [FS18] covers them pedagogically:

| Topic | [FS19] section | [FS18] section | Tracked in |
|---|---|---|---|
| Frobenius monoid definition | §2.2 Def 2.5 | §6.3.1 Def 6.52 | core audit |
| Hypergraph category definition | §2.3 Def 2.12 | §6.3.3 Def 6.60 | core audit |
| Cospan_C as hypergraph category | §2.3 Ex 2.14 | §6.3 Ex 6.61 | core audit |
| Cospan pushout composition | §1 | §6.2.5 Def 6.43–6.45 | core audit |
| Self-dual compact closed | §3.1 Prop 3.1 | §6.3 Prop 6.66 | core audit |
| Cospan ≅ free Frobenius | §3.2 Prop 3.8 | §6.3 Thm 6.58 | core audit |
| Operads (motivational) | §2.5 | §6.5 Rough Def 6.91 | this audit (formal) / core audit (motivational) |
| SMC definition | implicit | §4.4.3 Rough Def 4.45 | core audit (implicit) |

For all overlapping items, the [FS19] research paper provides the rigorous version. The [FS18] textbook provides the pedagogical introduction. catgraph core implements the [FS19] versions; this audit does not re-count them.

---

## Enrichment extension point: grammar, language, and magnitude

catgraph-applied stays in the `Set`-enriched (ordinary) categorical world: all hom-sets are plain Rust collections. The *enriched* refinement — where hom-objects live in a monoidal base `V` (e.g. `[0,1]`, `[0,∞]`, a tropical semiring, or a more general semiring) — is deliberately pushed one level up into a future sibling crate `catgraph-magnitude` (Phase 6). This section records the paper provenance and the cross-link so future readers can trace the design decision.

### Paper provenance for the enriched layer

- **[BTV21]** Bradley, Terilla, Vlassopoulos, *An enriched category theory of language: from syntax to semantics* (arXiv:2106.07890v2, 2021).
  Defines the syntax category `L` — objects = strings of tokens, hom-objects `L(x, y) = π(y|x) ∈ [0,1]` (extension probability), enrichment over `([0,1], ≥, ·, 1)`. The semantic category `L̂` is the enriched copresheaf category; Yoneda identifies each text with its representable `L(x, −)`. [FS18] §1.1 (enriched categories over a poset) + §2.4 (V-categories) are the relevant pedagogical anchors; [FS18] does not cover language or LLM applications.

- **[BV25]** Bradley, Vigneaux, *The magnitude of categories of texts enriched by language models* (arXiv:2501.06662v2, 2025).
  For an autoregressive LM with BOS `⊥`, EOS `†`, and cutoff, defines a `[0,1]`-enriched category `M`. Computes the magnitude function via the Leinster-Shulman Möbius construction:

  ```
  Mag(tM) = (t − 1) · Σ_{x ∈ ob(M) \ T(⊥)} H_t(p_x)  +  #(T(⊥)),   t > 0
  ```

  where `H_t` is the `t`-logarithmic (Tsallis) entropy and `T(⊥)` is the set of terminating strings. Recovers a sum of Shannon entropies at `d/dt Mag|_{t=1}`. Expresses magnitude as the Euler characteristic of magnitude homology; gives `MH_0` and `MH_1` explicitly.

### Why the enriched layer is not in catgraph-applied v0.1.0

1. **Scope discipline.** catgraph-applied v0.1.0 covers [FS18] Ch 4–6 in the unenriched setting. Enrichment adds a parallel axis (the base `V`) that belongs in a dedicated crate rather than leaking into every existing type signature.
2. **Semiring machinery is substantial.** `Semiring`, `WeightedCospan<Q>`, magnitude, Möbius, Tsallis entropy, `[0,∞]`-metric-space view, tropical semiring — enough surface area for a standalone crate.
3. **Magnitude is physics-application-grade.** BV25's closed-form magnitude formula is the first application that promotes "enriched cospans" from a nice abstraction to a concrete numerical invariant with a correctness anchor. It deserves its own release cadence and example suite.
4. **Agent-coalition application is pending downstream framework evaluation.** The Phase 6 design needs to settle on an agent-framework substrate before locking the `Semiring + WeightedCospan` API. Doing this inside catgraph-applied would couple the applied-CT audit to an unsettled downstream design.

### Grammar without external grammar input

A central claim of [BTV21] is that **grammatical and semantic content arise from the enriched structure alone** — no externally-supplied grammar is needed. This contrasts with DisCoCat (Coecke-Sadrzadeh-Clark), which takes a pregroup grammar as input.

For catgraph-applied, this implies a non-obvious design insight: **the `wiring_diagram` + operadic substitution layer, once lifted to `[0,1]`-enriched named cospans, can express compositional grammar directly.** Concretely:

- Ports on inner/outer circles carry probabilistic hom-annotations.
- Operadic substitution (`WiringDiagram::substitute`) composes annotations multiplicatively (the enrichment base `([0,1], ·)` is the composition monoid).
- The free `[0,1]`-enriched monoidal category on a signature of port types is what BTV21's syntax category `L` is, up to choice of generators.

No changes to catgraph-applied v0.1.0 code; the point is that the *same* operadic substitution machinery already in `wiring_diagram.rs` becomes a grammatical engine once weighted. This is the bridge between [FS18] §6.5 (operads and operad algebras, unenriched) and [BTV21] (`[0,1]`-enriched language categories).

### Time-step discretization as a functor

The Wolfram-physics modules in `catgraph-physics` (`multiway::evolution_graph`, `hypergraph::evolution_cospan`) already implement a specific instance of the "continuous → discrete" functorial pattern: multiway evolution (branching, generative) is discretized into cospan chains (sequential, observational). See [`catgraph-physics`](../../catgraph-physics/src/multiway/evolution_graph.rs) module-header rustdoc (added in Phase 5.1) for the CT framing.

The same `C → D` functor pattern appears in [Gor23] (Gorard's functorial irreducibility) and in the Mamba-style state-space-model discretization (exponential-trapezoidal, bilinear transform, zero-order hold). Three different domains, one compositional pattern. [BV25]'s magnitude is a candidate quantitative measure of how much information `D` carries about `C`.

### Phase 6 target: `catgraph-magnitude`

The planned `catgraph-magnitude` sibling crate will provide `Semiring`,
`WeightedCospan<Q>`, `LmCategory`, and a `magnitude()` functional as v0.1.0
targets, anchored to [BTV21] (arXiv:2106.07890) and [BV25] (arXiv:2501.06662).
The magnitude functional `Mag(tA)` with Tsallis parameter `t` gives a
quantitative diversity invariant; `t=1` recovers Shannon entropy, `t=2`
recovers collision probability, and `t → ∞` recovers cardinality.

No catgraph-applied code change is required for any of the above. The
enrichment layer stays in the Phase 6 sibling crate.
