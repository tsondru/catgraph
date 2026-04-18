# catgraph-applied

Applied category theory extensions for [catgraph](../catgraph). Anchored to [Fong & Spivak, *Seven Sketches in Compositionality* (arXiv:1803.05316v3, 2018)](https://arxiv.org/abs/1803.05316), Chapters 4–6.

## Overview

This crate packages applied-CT modules that build on catgraph's strict Fong-Spivak 2019 core but are not part of the 2019 paper's numbered content. It is the applied-CT complement to the F&S core crate.

## Modules

| Module | Purpose |
|---|---|
| `decorated_cospan` | Generic `Decoration` trait + `DecoratedCospan<Lambda, D>` realizing F&S Def 6.75 + Thm 6.77 |
| `wiring_diagram` | Operadic substitution built on named cospans |
| `petri_net` | Place/transition nets with cospan bridge, firing, reachability, parallel/sequential composition, `HypergraphCategory` impl, `PetriDecoration` bridge to `DecoratedCospan` |
| `temperley_lieb` | Temperley-Lieb / Brauer algebra via perfect matchings |
| `linear_combination` | Formal linear combinations over a coefficient ring |
| `e1_operad` | Little-intervals operad (E₁) |
| `e2_operad` | Little-disks operad (E₂) |

## Dependency on catgraph

Every module depends on catgraph's public API:

- `Cospan`, `NamedCospan`, `Span`, `Rel` — pushout/pullback composition
- `Frobenius` generators — operadic composition of SMCs (Prop 3.8)
- `HypergraphCategory` trait — target for semantic functors
- `Operadic` trait — abstract substitution interface (concrete impls live here)
- `compact_closed` cup/cap — string-diagram rewriting (TL, wiring)

## Paper alignment

See [`docs/SEVEN-SKETCHES-AUDIT.md`](docs/SEVEN-SKETCHES-AUDIT.md) for the section-by-section Seven Sketches coverage audit (Chapters 4–6, 56 items tracked). Cross-linked from [`../catgraph/docs/FONG-SPIVAK-AUDIT.md`](../catgraph/docs/FONG-SPIVAK-AUDIT.md) "Reconciliation" section.

## Release history

- **v0.1.0** — initial extraction from catgraph core (Petri nets, operads, Temperley-Lieb, wiring diagrams, linear combinations)
- **v0.2.0** — Seven Sketches audit published (`docs/SEVEN-SKETCHES-AUDIT.md`)
- **v0.3.0** — Tier 1 gap closures: `decorated_cospan` module, `HypergraphCategory` impl for `PetriNet`, Circuit EdgeSet example.
- **v0.3.1** — Tier 1.1 closures: `DecoratedCospan::compose` invokes `D::pushforward` through the pushout quotient (correct series composition for apex-index-carrying decorations like Circuit EdgeSet); `PetriNet::permute_side` implements braiding directly on the transition sequence; `Transition::relabel` deduplicates arcs when the quotient collapses distinct places onto the same target. Requires catgraph v0.11.3 for `Cospan::compose_with_quotient`.

## Build

```sh
cargo test -p catgraph-applied
cargo clippy -p catgraph-applied -- -W clippy::pedantic
```

## License

MIT.
