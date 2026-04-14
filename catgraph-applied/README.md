# catgraph-applied

Applied category theory extensions for [catgraph](../catgraph).

## Overview

This crate packages applied-CT modules that build on catgraph's strict Fong-Spivak 2019 core but are not part of the 2019 paper's numbered content. It is the applied-CT complement to the F&S core crate.

## Modules

| Module | Purpose |
|---|---|
| `wiring_diagram` | Operadic substitution built on named cospans |
| `petri_net` | Place/transition nets with cospan bridge, firing, reachability, parallel/sequential composition |
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

## Roadmap

A specific F&S applied CT paper (or equivalent) will be added to `docs/` to anchor the design. An audit document (`docs/FONG-SPIVAK-APPLIED-AUDIT.md`) will follow, paralleling `catgraph/docs/FONG-SPIVAK-AUDIT.md`.

See the workspace plan at `../.claude/refactor/current-plan.md` (Phase 5).

## Build

```sh
cargo test -p catgraph-applied
cargo clippy -p catgraph-applied -- -W clippy::pedantic
```

## License

MIT.
