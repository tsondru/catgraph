# catgraph-magnitude

Magnitude of enriched categories for the [catgraph](https://github.com/tsondru/catgraph)
workspace. Anchored to Bradley & Vigneaux, *Magnitude of Language Models* (2025).

**Status:** Phase 6A.0 scaffold (v0.1.0-dev). Implementation populates in subsequent commits.

## Scope (v0.1.0)

- `WeightedCospan<Λ, Q>` — `catgraph::Cospan<Λ>` decorated with per-edge weights in a rig `Q`.
- `tsallis_entropy(p, t)` — `H_t(p) = (1 − Σ pᵢᵗ)/(t−1)` with Shannon special case at `|t−1| < 1e-6`.
- `mobius_function<Q: Ring>(space)` — Möbius inversion `ζ · μ = I` per Leinster-Shulman §2.
- `magnitude<Q: Ring>(space, t)` — magnitude via Möbius sum.
- `LmCategory` — materialized language-model transition table with `Mag(tM)` per BV 2025 Thm 3.10.

## Acceptance criteria for v0.1.0

1. **BV 2025 Thm 3.10:** `Mag(tM) = (t−1) · Σ H_t(p_x) + #(T(⊥))` verified to machine precision on a hand-computed 3-state LM.
2. **BV 2025 Cor 3.14:** `d/dt Mag|_{t=1} = Σ H(p_x)` verified by central finite difference (`h = 1e-4`) to 1e-6.

## v0.1.0 algebraic scoping

Möbius inversion via Gaussian elimination on `Matrix<Q>` requires `Q` to have additive inverses (a **ring**, not merely a rig). v0.1.0 exposes a thin `Ring` super-trait over `Rig` and restricts `mobius_function` / `magnitude` to `Q: Ring`. `F64Rig` satisfies `Ring`; `BoolRig`, `UnitInterval`, `Tropical` do not.

A chain-sum `mobius_function_via_chains<Q: Rig>` per Leinster-Shulman §2 is deferred to v0.2.0.

## License

MIT — same as the catgraph workspace.
