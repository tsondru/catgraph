# PetriNet Design Spec

**Issue:** #1 — Petri net type on Cospan<Lambda>
**Date:** 2026-04-04
**Module:** `src/petri_net.rs` (catgraph core)

---

## Overview

A place/transition Petri net built on catgraph's cospan semantics. Places are Lambda-typed, transitions connect pre-places to post-places with weighted arcs, and markings assign token counts to places. Firing is pure (returns new marking). Composition connects to the cospan infrastructure for compositional Petri net construction.

Two-layer design:
1. **Static structure** — places, transitions, arc weights, structural queries
2. **Cospan bridge** — bidirectional conversion to/from `Cospan<Lambda>`, sequential and parallel composition mapping to pushout and monoidal product

## Types

### PetriNet

```rust
/// A place/transition Petri net with Lambda-typed places.
pub struct PetriNet<Lambda: Sized + Eq + Copy + Debug> {
    places: Vec<Lambda>,
    transitions: Vec<Transition>,
}
```

### Transition

```rust
/// A single transition: pre-set and post-set as weighted arcs over place indices.
pub struct Transition {
    pre: Vec<(usize, u64)>,   // (place_index, weight) — input arcs
    post: Vec<(usize, u64)>,  // (place_index, weight) — output arcs
}
```

### Marking

```rust
/// Token assignment: place index → count. Sparse (only nonzero entries stored).
pub struct Marking {
    tokens: HashMap<usize, u64>,
}
```

### PetriNetError

```rust
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PetriNetError {
    #[error("transition {index} not enabled under current marking")]
    NotEnabled { index: usize },
    #[error("transition index {index} out of bounds (net has {count} transitions)")]
    TransitionOutOfBounds { index: usize, count: usize },
    #[error("sequential composition failed: {message}")]
    CompositionFailed { message: String },
}
```

## API Surface

### Construction

- `PetriNet::new(places, transitions)` — construct from explicit places and transitions
- `PetriNet::from_cospan(cospan)` — single-transition net from a cospan (left leg multiplicities = pre-weights, right leg = post-weights)
- `Marking::new()` — empty marking
- `Marking::from_vec(tokens)` — from `Vec<(usize, u64)>` pairs
- `Marking::set(place, count)` / `Marking::get(place)` — accessors

### Firing

- `enabled(&self, marking) -> Vec<usize>` — transitions that can fire
- `fire(&self, transition, marking) -> Result<Marking, PetriNetError>` — pure, returns new marking
  - Enabled when `marking[p] >= pre(t, p)` for all input places
  - New marking: subtract pre-weights, add post-weights

### Reachability

- `reachable(&self, marking, max_depth) -> Vec<Marking>` — BFS over marking graph, bounded
- `can_reach(&self, initial, target, max_depth) -> bool` — short-circuits on first match

### Structural Queries

- `place_count()`, `transition_count()`
- `source_places()` — places with no input arcs from any transition
- `sink_places()` — places with no output arcs to any transition
- `arc_weight_pre(place, transition) -> u64` — zero if no arc
- `arc_weight_post(place, transition) -> u64` — zero if no arc

### Cospan Bridge

- `transition_as_cospan(&self, transition) -> Cospan<Lambda>` — single transition as cospan
- `from_cospan(cospan) -> Self` — construct single-transition net from cospan

### Composition

- `parallel(&self, other) -> Self` — disjoint union (maps to `Cospan::monoidal`)
- `sequential(&self, other) -> Result<Self, PetriNetError>` — identify output/input boundary places by Lambda match (maps to cospan pushout composition)

## Design Decisions

**Arc weights explicit, not implicit.** Transitions store `Vec<(usize, u64)>` rather than encoding weights as cospan leg multiplicity. Cleaner user API; cospan bridge handles conversion.

**Marking is sparse.** `HashMap<usize, u64>` rather than `Vec<u64>` — most places are empty in typical nets.

**Firing is pure.** `fire()` returns a new `Marking`, matching catgraph's style where `compose()` returns a new value.

**Reachability is bounded.** `max_depth` parameter prevents divergence on unbounded nets.

**Sequential composition uses Lambda matching.** Post-places of the first net (sink places with no outgoing arcs) are identified with pre-places of the second net (source places with no incoming arcs) when their Lambda values are equal. Matched places are merged into one; unmatched boundary places remain. This mirrors cospan pushout semantics and is the Baez-Master open Petri net approach.

## Relationship to Fong-Spivak Roadmap

The Petri net type connects to the catgraph design doc's gap analysis:

- **Cospan-algebra (§2.1):** A marking is a cospan-algebra element — a function from the places set to token counts. Future: the marking functor could be formalized as a lax monoidal functor from `Cospan_Lambda` to `Set`.
- **Composition (§3.2):** Sequential/parallel composition of Petri nets corresponds to cospan composition/monoidal product, demonstrating `Cospan_Lambda` as the free hypergraph category.
- **Magnitude enrichment (Session 3):** Token weights generalize to semiring-valued markings (see below).

## Future: Colored Tokens

The current design uses `u64` token counts (place/transition nets). Colored Petri nets generalize to typed tokens where each place holds a multiset of colored values, arcs carry guard/filter functions, and firing depends on token color matching. The `Marking` type could be generalized to `Marking<Token>` with `Token = u64` as the default. Deferred — integer markings cover chemical reactions, workflows, and protocol verification.

## Future: Token Weights and Magnitude Enrichment

Connects to Session 3 of the Fong-Spivak roadmap. Token counts are a degenerate case of weighted tokens where weights live in the semiring `(N, +, ×)`. Generalizations:

- **Stochastic Petri nets:** Transition rates as weights in `(R≥0, +, ×)`. Firing probability proportional to rate.
- **Continuous Petri nets:** Real-valued fluid levels in `(R≥0, +, ×)` instead of discrete token counts.
- **Timed Petri nets:** Duration weights in `(R≥0, max, +)` for scheduling analysis.

All fit the magnitude enrichment story: a `WeightedPetriNet<Lambda, W>` where `W: Semiring` enriches the token game with weights that propagate multiplicatively along fired paths and additively at merges — the weighted composition rule from the design doc (§5). The marking becomes `HashMap<usize, W>` and firing applies semiring operations instead of integer arithmetic.

This bridges to the Bradley-Vigneaux magnitude formula: the magnitude of a Petri net's reachability graph, enriched by transition rates, computes a Tsallis entropy over the space of reachable states.

## Testing Strategy

- **Unit tests** in `src/petri_net.rs`: construction, enabled, fire, arc_weight queries
- **Integration tests** in `tests/petri_net.rs`:
  - Chemical reaction nets (H2+O2→H2O, two-step synthesis, Haber process — mirror existing catgraph-surreal domain tests)
  - Reachability (dining philosophers deadlock, producer-consumer bounded buffer)
  - Composition (sequential pipeline, parallel independence)
  - Cospan roundtrip (PetriNet → cospan → PetriNet preserves structure)
- **Example** in `examples/petri_net.rs`: combustion reaction, marking evolution, reachability check
