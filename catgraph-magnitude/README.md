# catgraph-magnitude

Magnitude of enriched categories for the [catgraph](https://github.com/tsondru/catgraph) workspace.
Anchored to Bradley & Vigneaux, *[Magnitude of Language Models](https://arxiv.org/abs/2501.06662)* (2025).

**Status:** v0.1.0 (Phase 6A complete; first publishable release).

## What

A pure-math Rust crate for computing the magnitude `Mag(tM)` of an enriched category over a rig
(BV 2025 §3). The headline use case is BV 2025's language-model magnitude, where `Mag(tM)` decomposes
via Tsallis q-entropy into a per-state diversity indicator that recovers Shannon entropy at `t = 1`
(BV 2025 Rem 3.11).

## Quickstart

```rust
use catgraph_magnitude::{LmCategory, magnitude::tsallis_entropy};

let mut m = LmCategory::new(vec!["⊥".into(), "⊥a".into(), "⊥a†".into()]);
m.add_transition("⊥", "⊥a", 1.0);
m.add_transition("⊥a", "⊥a†", 1.0);
m.mark_terminating("⊥a†");

let mag = m.magnitude(2.0).unwrap();
println!("Mag(2M) = {mag:.6}");   // 1.000000 (deterministic chain)
```

## v0.1.0 acceptance gate

Two BV 2025 verifications must pass for any v0.1.0 tag:

1. **Prop 3.10 closed form** — `Mag(tM) = (t−1) · Σ H_t(p_x) + #(T(⊥))` to `1e-9` on a
   hand-computed 4-state LM (`A = {a}, N = 1`; states `⊥, ⊥a, ⊥†, ⊥a†`).
2. **Rem 3.11 Shannon recovery** — `d/dt Mag|_{t=1} = Σ H(p_x)` by central finite difference
   (`h = 1e-4`) to `1e-6`.

Current residuals: `0e0` (Prop 3.10) / `~6e-10` (Shannon FD) respectively.

## API surface (v0.1.0)

| Symbol | Paper anchor | Notes |
|---|---|---|
| `LmCategory` | BV 2025 §3 | Materialized BYO-LM transition table |
| `magnitude<Q>(space, t)` | BV 2025 §3.5 Eq (7) | Möbius sum at scale `t` |
| `mobius_function<Q>(space)` | Leinster-Shulman §2 | `ζ⁻¹` via Gaussian elimination |
| `tsallis_entropy(p, t)` | BV 2025 Prop 3.10 / Tsallis 1988 | Shannon special case at `\|t−1\| < 1e-6` |
| `WeightedCospan<Λ, Q>` | F&S 2019 §1 + BV 2025 §3 | Cospan with per-edge rig weights |
| `LawvereMetricSpace<T>` (re-export) | Lawvere 1973 | Asymmetric metric space |
| `Rig`, `Ring`, `BoolRig`, `UnitInterval`, `Tropical`, `F64Rig` | F&S 2018 §5.3.1 | Re-exports + `Ring` super-trait |
| `TSALLIS_SHANNON_EPS` | numerical | Special-case threshold `1e-6` |

## Algebraic scoping

`mobius_function` and `magnitude` require `Q: Ring + Div + From<f64>` — i.e. a (commutative) field
for the v0.1.0 Gaussian-elimination implementation. Among the workspace's four concrete rigs, only
`F64Rig` qualifies. A chain-sum `mobius_function_via_chains<Q: Rig>` per Leinster-Shulman's
poset-walk formula is deferred to v0.2.0 to support `BoolRig` / `UnitInterval` / `Tropical` magnitude.

`tsallis_entropy` is `f64`-only; lifting it to a generic rig is non-trivial (the `0/0` limit form
requires real-valued epsilon comparisons and a `ln` operation).

## Numerical scoping

Public constant `TSALLIS_SHANNON_EPS = 1e-6` is the threshold below which `tsallis_entropy` returns
`-Σ pᵢ ln pᵢ` directly, avoiding catastrophic cancellation in the `(1 − Σ pᵢᵗ)/(t − 1) ≈ 0/0`
regime. The Rem 3.11 finite-difference step `h` MUST satisfy `h > TSALLIS_SHANNON_EPS`; the
recommended `h = 1e-4` gives ~2 decimal margin above the threshold while staying near `f64`'s
`ε^(1/3) ≈ 6e-6` truncation+roundoff optimum.

## Examples

```sh
cargo run --example lm_magnitude       # BV 2025 p.4 bounds on deterministic vs. uniform LMs
cargo run --example tsallis_shannon    # Shannon recovery to exactly-0 for δt < TSALLIS_SHANNON_EPS
cargo run --example mock_coalition     # 5-agent WeightedCospan + 3-agent LmCategory diversity demo
```

### `lm_magnitude`

Prints `Mag(tM)` at `t ∈ {0.5, 1.0, 2.0, 10.0, 1e6}` for two LMs and asserts four BV 2025 p.4
properties for `t ≥ 1`: lower bound `≥ #T(⊥)`, upper bound `≤ #ob(M)`, monotone non-decreasing,
and `t → ∞` limit `≈ #T(⊥)` within `1e-3`. Also verifies Prop 3.10 closed form agrees with the
Möbius-sum magnitude to `< 1e-9`.

### `tsallis_shannon`

Over 50 seeded random distributions of size 2–5, evaluates `tsallis_entropy(p, t)` at
`δt ∈ {1e-2, 1e-3, 1e-4, 1e-5, 1e-6, 1e-7}`. Asserts: (a) within the `TSALLIS_SHANNON_EPS`
special-case threshold the error is exactly zero; (b) at `δt = 1e-3` the worst error is `< 5e-3`.

### `mock_coalition`

Builds a 5-agent `WeightedCospan<&str, UnitInterval>` with asymmetric edge weights (including a
cycle), lifts to a Lawvere metric space, and prints the distance matrix. Then builds an acyclic
3-agent `LmCategory` sub-coalition and prints four magnitude-derived diversity indicators: `Mag(1.0)`,
`Mag(2.0)`, `Mag(1e6)`, and the Shannon-entropy finite-difference estimate. Demonstrates the API
without any transport deps (no SurrealDB, no tokio).

## Roadmap

- v0.1.0: closed-form magnitude on prefix-poset LMs (this crate).
- v0.2.0: rig-generic chain-sum Möbius via Leinster-Shulman explicit formula.
- Phase 6B (`catgraph-coalition`, external sibling): SurrealDB live-query agent transport, bridged
  via `Coalition::current_weighted_cospan() -> WeightedCospan<RecordId, UnitInterval>`.
- Phase 6C: BTV 2021 Yoneda copresheaves for grammar-from-enrichment.

## License

MIT.
