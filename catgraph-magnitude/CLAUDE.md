# catgraph-magnitude (BV 2025, v0.1.0)

Magnitude of enriched categories. Workspace member of [catgraph](../CLAUDE.md).

Anchored to Bradley & Vigneaux, *[Magnitude of Language Models](https://arxiv.org/abs/2501.06662)* (arXiv:2501.06662v2, 2025). Consumes the enrichment substrate from `catgraph-applied` v0.5.3+.

## Scope

Pure-math crate: no tokio, no serde, no rayon. All types are `Clone + Debug + PartialEq`.

- `weighted_cospan.rs` — `WeightedCospan<Λ, Q: Rig>` newtype over `catgraph::Cospan<Λ>` carrying per-edge rig weights. Type aliases `ProbCospan<Λ>`, `TropCospan<Λ>`. Specialized `into_metric_space` on `Q = UnitInterval` via `-ln π` embedding (Lawvere 1973).
- `ring.rs` — `Ring` super-trait over `Rig` with blanket impl over `Neg + Sub`. Required by Möbius inversion.
- `magnitude.rs` — `tsallis_entropy(p, t)` (BV 2025 Prop 3.10 / Tsallis 1988) with Shannon special case at `|t−1| < TSALLIS_SHANNON_EPS = 1e-6`. `mobius_function<Q: Ring>(space)` via Gaussian elimination. `magnitude<Q: Ring>(space, t)` via Möbius sum (BV 2025 §3.5 Eq 7).
- `lm_category.rs` — `LmCategory` materialized BYO-LM transition table. `magnitude(t)` lifts via prefix-extension semantics (BV 2025 §2.10–2.17).
- `lib.rs` — re-exports of enrichment substrate from `catgraph-applied`: `Rig`, `Ring`, `BoolRig`, `UnitInterval`, `Tropical`, `F64Rig`, `EnrichedCategory`, `HomMap`, `LawvereMetricSpace`, `MatR`.

**Out of scope:**
- Rig-generic chain-sum Möbius (`mobius_function_via_chains<Q: Rig>`) — deferred to v0.2.0
- Agent transport (SurrealDB RELATE, tokio live-queries) — see `catgraph-coalition` (Phase 6B external sibling)
- BTV 2021 Yoneda copresheaves — deferred to Phase 6C

## Paper alignment

Anchored to BV 2025 (arXiv:2501.06662v2). v0.1.0 acceptance gate:

1. **Prop 3.10 closed form** — `Mag(tM) = (t−1)·Σ H_t(p_x) + #(T(⊥))` to `0e0` (exact) on 4-state hand-computed LM at `t ∈ {0.5, 1.5, 2.0, 5.0}`.
2. **Rem 3.11 Shannon recovery** — `d/dt Mag|_{t=1} = Σ H(p_x)` by central FD (`h = 1e-4`) to `~6e-10`.

Both tests live in `tests/bv_2025_acceptance.rs` and pass at v0.1.0.

## Build

```sh
cargo test -p catgraph-magnitude
cargo clippy -p catgraph-magnitude -- -W clippy::pedantic
cargo test -p catgraph-magnitude --examples
cargo bench -p catgraph-magnitude --no-run
```

## Dependencies

- `catgraph = "0.12"` (path dep during development)
- `catgraph-applied = "0.5"` — requires v0.5.3+ for `F64Rig` ring + field ops
- `num` (workspace dep)
- `proptest`, `criterion` (dev only)
