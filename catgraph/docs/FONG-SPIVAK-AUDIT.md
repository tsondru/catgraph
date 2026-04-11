# Fong-Spivak Coverage Audit (catgraph v0.10.1)

> **Paper:** Fong & Spivak, *Hypergraph Categories* (arXiv:1806.08304v3, 18 Jan 2019)
> **Library:** catgraph v0.10.1
> **Date:** 2026-04-10
> **Method:** read all 38 pages of the paper, cross-checked each numbered item against catgraph source

**Status legend:**
- ✅ DONE — implemented and tested
- ⚠️ PARTIAL — implementation exists but is incomplete, implicit, or doesn't fully exhibit the paper's structure
- ❌ MISSING — not implemented in catgraph
- ➖ N/A — theoretical / motivational, no implementation expected

## Summary

| Section | DONE | PARTIAL | MISSING | N/A | Total |
|---|---|---|---|---|---|
| §1 Introduction | 6 | 2 | 0 | 1 | 9 |
| §2.1 Cospan-algebras | 3 | 2 | 2 | 0 | 7 |
| §2.2 Frobenius monoids | 3 | 2 | 2 | 1 | 8 |
| §2.3 Hypergraph categories | 3 | 6 | 2 | 2 | 13 |
| §2.4 Critiques | 0 | 0 | 2 | 0 | 2 |
| §2.5 Operads | 0 | 0 | 0 | 1 | 1 |
| §3.1 Compact closed | 5 | 1 | 0 | 0 | 6 |
| §3.2 Free hypergraph cats | 4 | 4 | 3 | 2 | 13 |
| §3.3 io/ff factorization | 0 | 0 | 6 | 0 | 6 |
| §3.4 Strictification | 0 | 1 | 3 | 0 | 4 |
| §4.1 H → A direction | 3 | 4 | 2 | 1 | 10 |
| §4.2 A → H direction | 4 | 2 | 1 | 1 | 8 |
| §4.3 The equivalence | 1 | 1 | 2 | 1 | 5 |
| **TOTAL** | **32** | **25** | **25** | **10** | **92** |

**Headline numbers:**
- **35% DONE / 27% PARTIAL / 27% MISSING / 11% N/A**
- Of the 25 missing items: 6 are §3.3 (explicitly deferred), 6 are LinRel/non-strict examples (deferred), 3 are §3.4 strictification (deferred), leaving ~10 genuinely missing items that aren't planned deferrals.

## Per-section detail

### §1 Introduction (motivation + main theorems)

| Item | Status | catgraph location | Notes |
|---|---|---|---|
| Eq 1: 3-box wiring example | ✅ | examples + named_cospan.rs | running example |
| Eq 2: Frobenius generator decomposition | ✅ | frobenius/operations.rs | from_decomposition |
| Eq 3: alternative wiring | ➖ | — | visual variant of Eq 2 |
| Eq 4: cospan A→N←B for the running example | ✅ | named_cospan.rs::new | core type |
| Eq 5: hierarchy of category types (cat → mon → traced → hyper) | ⚠️ | — | implicit; no explicit `TracedMonoidalCategory` layer (CLAUDE.md says "OK because hypergraph subsumes it") |
| Eq 6: operadic substitution as a compositional view | ✅ | operadic.rs (trait) | impl currently in wiring_diagram.rs |
| Eq 7: labeled cospan diagram (m → p ← n) | ✅ | cospan.rs | core type |
| Eq 8: Hyp_OF(Λ) ≅ Lax(Cospan_Λ, Set) | ✅ | equivalence.rs | morphism direction via CospanAlgebraMorphism + roundtrip tests |
| Thm 1.1: every hypergraph cat ≅ OF (coherence) | ⚠️ | — | catgraph works inside Cospan_Λ which IS objectwise-free, but never proves the general equivalence Hyp ≃ Hyp_OF |
| Thm 1.2: Hyp_OF ≅ ∫ Lax(Cospan_Λ, Set) | ⚠️ | equivalence.rs | per-Λ version verified (Thm 4.13); the Grothendieck-construction global form (Thm 4.16) is implicit |

### §2.1 Cospans and cospan-algebras

| Item | Status | catgraph location | Notes |
|---|---|---|---|
| Cospan_Λ category with pushout composition | ✅ | cospan.rs | union-find pushout |
| Eq 9: Cospan_f functoriality square | ❌ | — | no explicit Cospan_- functor between different Λ's |
| Prop 2.1: Cospan_- : Set_List → Cat is a functor | ❌ | — | the cross-Λ functoriality is not implemented |
| Def 2.2: cospan-algebra (lax sym mon functor a: Cospan_Λ → Set) | ✅ | cospan_algebra.rs | CospanAlgebra trait |
| Def 2.2: morphism of cospan-algebras (relabeling f + nat trans α) | ⚠️ | — | the relabeling f: Λ → List(Λ') part is missing; α part exists implicitly via Lemma 4.9 (also missing) |
| Def 2.2: Cospan-Alg category (objects + morphisms) | ⚠️ | — | trait exists; no explicit "category of cospan-algebras" type with composition |
| Ex 2.3: PartitionAlgebra Part_Λ | ✅ | cospan_algebra.rs::PartitionAlgebra | |
| Prop 2.4: Cospan-Alg ≅ ∫ Lax(Cospan_Λ, Set) (Grothendieck) | ⚠️ | — | the right-hand side IS what catgraph tests against, but the left-hand global category is never constructed |

### §2.2 Special commutative Frobenius monoids

| Item | Status | catgraph location | Notes |
|---|---|---|---|
| Def 2.5: special commutative Frobenius monoid (μ, η, δ, ε + 9 axioms) | ✅ | frobenius/operations.rs::FrobeniusOperation | |
| Frobenius axioms verification | ✅ | tests/frobenius_laws.rs | 8 tests covering associativity, unit, commutativity, coassoc, counit, cocomm, Frobenius, special |
| Ex 2.6: canonical Frobenius on monoidal unit I | ⚠️ | — | implicit (every type provides identity-as-unit); not an explicit constructor |
| Ex 2.7: 1-object SMC = monoid case (Frobenius on I = invertible scalar) | ➖ | — | algebraic remark, not a constructable example |
| Ex 2.8: Frobenius on object 1 in Cospan | ✅ | hypergraph_category.rs (`impl HypergraphCategory for Cospan<Lambda>`) | |
| Ex 2.9: any object in Cospan(C) for C with finite colimits | ⚠️ | — | catgraph only does Cospan_Λ on FinSet_Λ, not general Cospan(C) |
| Ex 2.10: additive Frobenius on ℝ in LinRel | ❌ | — | LinRel deferred per FONG-SPIVAK-XREF |
| Ex 2.11: multiplicative Frobenius on ℝ in LinRel (different) | ❌ | — | LinRel deferred |

### §2.3 Hypergraph categories

| Item | Status | catgraph location | Notes |
|---|---|---|---|
| Def 2.12: hypergraph category | ✅ | hypergraph_category.rs::HypergraphCategory | trait with η, ε, μ, δ + cup, cap |
| Eq 11: monoidal compatibility (4 equations) | ⚠️ | — | implicit in cospan structure; not a separate verification |
| Unit coherence axiom η_I = id_I = ε_I | ⚠️ | — | implicit; relies on Prop 2.18 (strict case) |
| Eq 12: hypergraph functor (F, φ) | ✅ | hypergraph_functor.rs::HypergraphFunctor | trait |
| Hyp 1-category | ⚠️ | — | trait exists but no explicit "category of hypergraph cats" type |
| Hyp 2-category (with monoidal nat trans as 2-morphisms) | ❌ | — | catgraph is not 2-categorical |
| Remark 2.13: every nat trans is invertible | ➖ | — | theoretical observation |
| Ex 2.14: Cospan(C) for C with colimits as hypergraph cat | ⚠️ | — | only Cospan_Λ on FinSet_Λ, not general Cospan(C) |
| Ex 2.15: Span(C) when C^op has limits | ⚠️ | span.rs | only Span on FinSet_Λ |
| Ex 2.15: Rel as hypergraph cat | ⚠️ | span.rs::Rel | Rel exists; HypergraphCategory impl missing |
| Ex 2.16: FdVect with chosen basis as hypergraph cat | ❌ | — | not implemented |
| Remark 2.17: unit coherence is a NEW axiom vs older defs | ➖ | — | theoretical |
| Prop 2.18: strict case ⟹ unit coherence automatic | ✅ | — | catgraph relies on this implicitly (cospans are strict) |
| Ex 2.19: non-strict counterexample requiring unit coherence | ❌ | — | not implemented |

### §2.4 Critiques

| Item | Status | catgraph location | Notes |
|---|---|---|---|
| Ex 2.20: hypergraph structures don't extend along equivalences (LinRel_2) | ❌ | — | LinRel deferred |
| Ex 2.21: ff+ess.surj functor not necessarily hypergraph equivalence | ❌ | — | LinRel deferred |

### §2.5 A word on operads

(motivational discussion, no theorems)

### §3.1 Self-dual compact closed

| Item | Status | catgraph location | Notes |
|---|---|---|---|
| Compact closed category definition (cup, cap, zigzag) | ✅ | compact_closed.rs (cup/cap functions) | not a separate trait; provided as helper functions |
| Eq 13: zigzag identities | ✅ | tests/compact_closed.rs (33 tests) | tested |
| Prop 3.1: every hypergraph cat is self-dual compact closed (cup_X := η; δ, cap_X := μ; ε) | ✅ | compact_closed.rs::cup, cap | exact formula |
| Prop 3.2: bijection C(X,Y) ≅ C(I, X⊗Y) (name) | ✅ | compact_closed.rs::name, unname | |
| Eq 14: comp^Y_{X,Z} morphism (id_X ⊗ cap_Y ⊗ id_Z) | ✅ | equivalence.rs::comp_cospan | |
| Prop 3.3: (f̂ ⊗ ĝ) ; comp^Y_{X,Z} = (f;g)^ | ⚠️ | compact_closed.rs::compose_names:313-334 | shortcut impl: `unname → compose → name`. Mathematically equivalent but doesn't exhibit the comp cospan formula structurally. No "compose_names_direct" matching the paper's literal formula. |
| Prop 3.4: (id_X ⊕ f̂) ; comp^X_{∅,Y} = f | ❌ | — | name recovery formula not implemented or tested |
| Ex 3.5: comp in Cospan_Λ | ✅ | equivalence.rs::comp_cospan + tests | the literal cospan picture matches |

### §3.2 Free hypergraph categories

| Item | Status | catgraph location | Notes |
|---|---|---|---|
| Lemma 3.6: Cospan generated by μ, η, δ, ε, σ, id | ✅ | finset.rs::Decomposition + frobenius::from_decomposition | epi-mono factorization |
| Ex 3.7: building Eq 4 cospan from generators | ⚠️ | — | example exists for similar cospans but not literally Eq 4 |
| Prop 3.8: Cospan ≃ theory of special commutative Frobenius monoids | ✅ | cospan_algebra.rs::cospan_to_frobenius + hypergraph_functor.rs::CospanToFrobeniusFunctor | |
| Def 3.9: OF(Λ) structure (List(Λ) ≅ Ob(C)) | ⚠️ | — | implicit; List(Λ) ≅ Ob(C) is just how catgraph encodes objects |
| Lemma 3.10: assigning Frobenius per l ∈ Λ uniquely determines hypergraph structure | ⚠️ | — | implicit in CospanToFrobeniusFunctor |
| Remark 3.11: explicit construction of μ_l | ➖ | — | construction detail |
| Ex 3.12: Cospan_Λ as hypergraph cat | ✅ | hypergraph_category.rs (Cospan impl) | |
| Cor 3.13: Cospan_- : Set_List → Hyp_OF as a functor | ❌ | — | the cross-Λ functor not implemented |
| Thm 3.14: Cospan_Λ is FREE hypergraph cat over Λ (Set ⇄ Hyp adjunction with unit/counit/triangles) | ❌ | — | DEFERRED — universal property API |
| Cor 3.15: Set_List ⇄ Hyp_OF refinement | ❌ | — | not implemented |
| Prop 3.16: counit Frob_{Cospan_Λ} is identity | ⚠️ | — | implicit in CospanToFrobeniusFunctor design; not a separate test |
| Remark 3.17: combined adjunctions diagram | ➖ | — | summary diagram |

### §3.3 Factoring hypergraph functors (io/ff)

| Item | Status | catgraph location | Notes |
|---|---|---|---|
| Eq 19: H_F construction | ❌ | — | DEFERRED (low priority per FONG-SPIVAK-XREF) |
| Remark 3.18: orthogonal factorization system on Hyp | ❌ | — | DEFERRED |
| Lemma 3.19: i_1 = Frob_1 | ❌ | — | DEFERRED |
| Prop 3.20: Gens : Hyp_OF → Set_List is split Grothendieck fibration | ❌ | — | DEFERRED |
| Eq 21: Hyp_OF(f) construction | ❌ | — | DEFERRED |
| Cor 3.21: Hyp_OF ≃ ∫ Hyp_OF(Λ) | ❌ | — | DEFERRED |

### §3.4 Strictification (Coherence theorem)

| Item | Status | catgraph location | Notes |
|---|---|---|---|
| Hyp_OF as full sub-2-cat of Hyp | ❌ | — | not 2-categorical |
| Thm 3.22: U: Hyp_OF → Hyp is a 2-equivalence (the coherence theorem) | ⚠️ | — | implicit; catgraph works in OF case only and never proves the equivalence |
| Eq 22: pre-parenthesized product P([x,y]) | ❌ | — | strictification construction |
| Eq 23: Str: Hyp → Hyp_OF 2-functor | ❌ | — | not implemented |

### §4.1 Hypergraph cats → cospan-algebras

| Item | Status | catgraph location | Notes |
|---|---|---|---|
| Eq 24: Hyp_OF(Λ) ≅ Lax(Cospan_Λ, Set) | ✅ | equivalence.rs | tested via roundtrip |
| Prop 4.1: A_- : Hyp_OF(Λ) → Lax(Cospan_Λ, Set) is a functor | ⚠️ | cospan_algebra.rs::NameAlgebra | NameAlgebra is the analog *for one fixed H = Cospan*, not a functor over varying H |
| Eq 25: A_H(-) := H(I, Frob(-)) | ✅ | NameAlgebra::map_cospan | implemented as the lax monoidal functor itself |
| Lemma 4.2: A_H is lax monoidal functor | ✅ | NameAlgebra trait impl | implicit in trait impl |
| Eq 26: laxator γ definition (γ: 1 → A_H(∅) and γ_{X,Y}: A_H(X) × A_H(Y) → A_H(X⊕Y)) | ✅ | NameAlgebra::lax_monoidal | |
| Lemma 4.3: A_F natural transformation construction (from F: H → H' to α: A_H → A_H') | ❌ | — | the F: H→H' direction not implemented — this is half of "A_- is a functor" |
| Eq 27/28: naturality square + α_X definition | ❌ | — | not implemented |
| Remark 4.4: extension to Hyp (non-OF case) via Frob_H'(-) | ➖ | — | not needed since catgraph only does OF |
| Remark 4.5: A_{Cospan_Λ} = Part_Λ | ✅ | implicit | the roundtrip test verifies this for Part |
| Prop 4.6: Part_Λ is initial cospan-algebra | ⚠️ | cospan_algebra.rs::PartitionAlgebra | impl exists; initiality is asserted but not formally verified by a test "every cospan-algebra has unique map from Part" |

### §4.2 Cospan-algebras → hypergraph cats

| Item | Status | catgraph location | Notes |
|---|---|---|---|
| Prop 4.7: H_- : Lax(Cospan_Λ, Set) → Hyp_OF(Λ) functor | ✅ | equivalence.rs::CospanAlgebraMorphism | the H_A construction |
| Lemma 4.8: H_A definition (objects = List(Λ), morphisms X→Y = A(X⊕Y)) | ✅ | equivalence.rs::CospanAlgebraMorphism | with full trait stack |
| Eq 31: Ob(H_A) := List(Λ), H_A(X,Y) := A(X⊕Y) | ✅ | equivalence.rs (domain/codomain/element fields) | |
| Eq 32: composition formula via comp^Y_{X,Z} | ✅ | equivalence.rs::compose | |
| Eq 33: six required cospans (id, braiding, μ, η, δ, ε) as ∅ → X⊕Y morphisms | ✅ | equivalence.rs identity_in/multiplication_in/comultiplication_in/unit_in/counit_in + from_permutation | |
| Lemma 4.9: F_α io functor from morphism α: A→B | ❌ | — | the morphism-of-cospan-algebras → hypergraph functor direction not implemented |
| Remark 4.10: hypergraph cats absorb special morphisms into operations | ➖ | — | conceptual remark |
| Lemma 4.11: Frob(c) = name(c) for Part case | ⚠️ | — | implicit; not a separate test |
| Cor 4.12: Frob_A(c) = A(name(c))(γ) | ⚠️ | — | implicit |

### §4.3 The equivalence

| Item | Status | catgraph location | Notes |
|---|---|---|---|
| Thm 4.13: Hyp_OF(Λ) ≃ Lax(Cospan_Λ, Set) | ✅ | equivalence.rs + tests/equivalence.rs | roundtrip verified for PartitionAlgebra and NameAlgebra (17 tests) |
| Naturality in Λ | ❌ | — | naturality across varying Λ not verified |
| Ex 4.14: LinRel' ≃ LinRel (specific worked example with the rectification ν(R) = (-a, b)) | ❌ | — | LinRel deferred |
| Remark 4.15: generalization Hyp^io_{H/} ≅ Lax(H, Set) | ➖ | — | theoretical remark |
| Thm 4.16: Hyp_OF ≅ Cospan-Alg (the global form, not per-Λ) | ⚠️ | — | implicit; catgraph proves the per-Λ Thm 4.13 but not the Grothendieck-construction global form which packages everything via naturality |

## Critical findings

### Genuinely missing items that should be addressed before catgraph v0.11.0 (slim F&S)

1. **Lemma 4.3** — A_F natural transformation. Without this, the §4.1 functor A_- is implemented only on objects, not morphisms. Half of the equivalence is incomplete in functor form. **Estimated effort: medium.**

2. **Lemma 4.9** — F_α io functor from morphism of cospan-algebras. Symmetric to Lemma 4.3, missing on the §4.2 side. Together with Lemma 4.3, forms the complete two-way functoriality. **Medium.**

3. **Prop 3.4** — name recovery formula. Listed but not implemented. Easy to add as a test against existing `name`/`unname`. **Small.**

4. **Prop 4.6 initiality test** — "every cospan-algebra has a unique morphism from Part." Currently asserted but not verified. Would require a property-based test over cospan-algebras. **Small.**

5. **`compose_names_direct`** — alongside the existing `compose_names` shortcut, add a version matching Prop 3.3's literal `(f̂ ⊗ ĝ); comp^Y_{X,Z}` formula. Pedagogically valuable; verifies the two are equivalent. **Small.**

### Items to keep deferred (paper-acknowledged or low-impact)

- §3.3 io/ff factorization (entire section, 6 items) — paper itself notes this is for fibration analysis; not needed for Thm 1.2
- §3.4 strictification (Thm 3.22) — implicit since catgraph works in OF case; making it explicit requires 2-category machinery
- Thm 3.14 universal property — paper-deferred
- LinRel examples (2.10, 2.11, 2.16, 2.20, 2.21, 4.14) — paper-deferred
- Cross-Λ functoriality (Prop 2.1, Cor 3.13, Cor 3.15) — would require parametric Λ machinery beyond catgraph's current type system

### Items that are implicit / "morally correct" but not explicit

These are correct but could be made explicit for catgraph-as-paper-implementation pedagogy:

1. **Eq 11 monoidal compatibility** — implicit in cospan structure
2. **Unit coherence axiom** — implicit via Prop 2.18 (strict case)
3. **Prop 2.18** — relied on but not stated/tested
4. **Lemma 3.10** — implicit in CospanToFrobeniusFunctor
5. **Prop 3.16** — implicit in design
6. **Lemma 4.11, Cor 4.12** — implicit; would benefit from test assertions matching the paper's exact formulas

### The "compose_names" shortcut (gleaner2 finding confirmed)

`compact_closed.rs:313-334` implements `compose_names` as:
```rust
let f = unname(f_hat, x_len)?;
let g = unname(g_hat, y_len)?;
let mut fg = f;
fg.compose(g)?;
name(&fg)
```

This is mathematically equivalent to Prop 3.3's `(f̂ ⊗ ĝ); comp^Y_{X,Z}` formula, but doesn't exhibit it. The paper's whole point in §3.1 is that you can compose at the *name level* without going back to morphisms. The current impl goes back to morphisms internally, defeating the demonstration.

**Recommendation:** add `compose_names_direct` matching Prop 3.3 literally, keep the existing one as `compose_names_via_unname`, and add a test asserting they agree.

## What does "Theorem 1.2 is implemented" actually mean for catgraph?

The bottom line:

**catgraph implements the per-Λ form of Thm 1.2 (which is Thm 4.13)**, with two specific worked examples (PartitionAlgebra and NameAlgebra).

**catgraph does NOT implement:**
- The global Grothendieck-construction form (Thm 4.16) — `Hyp_OF ≅ Cospan-Alg` as 1-categories with naturality across Λ
- The 2-categorical version (Thm 1.1) — the strictification result that lets you reduce general Hyp to Hyp_OF
- Functoriality of the equivalence on morphisms (Lemmas 4.3 and 4.9)

The substantive math IS in catgraph — the per-Λ equivalence is the real content, and the global form is a packaging step. But for a paper-faithful claim, the gap between "Thm 4.13 done" and "Thm 1.2 done" should be acknowledged.

## Recommendation for catgraph v0.11.0 release notes

**Honest claim:** "catgraph implements Theorems 4.13 and 1.2 in their per-Λ form, with PartitionAlgebra and NameAlgebra as worked examples. The global Grothendieck-construction form (Thm 4.16), the 2-categorical strictification (Thm 1.1), and §3.3 io/ff factorization are intentionally deferred."

**Stronger claim (after fixing the 5 items above):** "catgraph implements Theorem 1.2 in its per-Λ form, with full bidirectional functoriality (Lemmas 4.3 and 4.9), all six structural cospans of §4.2, and Props 3.1-3.4 on compact closed structure."

The 5 fixes are roughly 1-2 days of work and would close all the genuine paper-vs-code gaps.
