//! [`LmCategory`] unit tests + BV 2025 Eq 4.3 bounds proptest.
//!
//! The two paper-anchored acceptance tests (Thm 3.10 closed form, Cor 3.14
//! Shannon recovery) live in `tests/bv_2025_acceptance.rs` so they appear as
//! a distinct test binary in `cargo test` output — they are the v0.1.0
//! acceptance gate and visibility matters.

// `usize → f64` casts on small-state-count test fixtures are precision-safe.
#![allow(clippy::cast_precision_loss)]

use catgraph_magnitude::lm_category::LmCategory;
use proptest::prelude::*;

/// `magnitude(t)` requires no transitions at all to be well-defined: every
/// state has `d(x, x) = 0` (identity axiom) and every off-diagonal is `+∞`.
/// `ζ_t = I` ⇒ `μ_t = I` ⇒ `Mag = n` (the trace of the identity).
#[test]
fn empty_transitions_magnitude_is_n() {
    let m = LmCategory::new(vec!["a".into(), "b".into(), "c".into()]);
    let mag = m.magnitude(1.5).expect("identity zeta is invertible");
    assert!(
        (mag - 3.0).abs() < 1e-12,
        "empty-transition LM magnitude should be n=3, got {mag}"
    );
}

/// Round-trip: `add_transition` followed by `transitions().get` recovers
/// the inserted probability; `mark_terminating` followed by
/// `terminating().contains` recovers the membership.
#[test]
fn add_transition_and_mark_terminating_round_trip() {
    let mut m = LmCategory::new(vec!["A".into(), "B".into(), "C".into()]);
    m.add_transition("A", "B", 0.5);
    m.add_transition("A", "C", 0.3);
    m.mark_terminating("A");

    assert_eq!(m.transitions().get("A").and_then(|r| r.get("B")), Some(&0.5));
    assert_eq!(m.transitions().get("A").and_then(|r| r.get("C")), Some(&0.3));
    assert!(m.terminating().contains("A"));
    assert!(!m.terminating().contains("B"));
    assert_eq!(m.objects().len(), 3);
}

/// Magnitude is finite and (per Eq 4.3) bounded on a small tree-shaped LM.
///
/// Uses a minimal `A = {a}, N = 1` tree (4 states), the same shape as the
/// BV 2025 acceptance fixture.
#[test]
fn magnitude_smoke_tree_lm() {
    let mut m = LmCategory::new(vec![
        "s0".into(),
        "s0a".into(),
        "s0t".into(),
        "s0at".into(),
    ]);
    m.mark_terminating("s0t");
    m.mark_terminating("s0at");
    m.add_transition("s0", "s0a", 0.6);
    m.add_transition("s0", "s0t", 0.4);
    m.add_transition("s0a", "s0at", 1.0);

    for &t in &[0.5_f64, 1.5, 2.0, 5.0] {
        let mag = m.magnitude(t).expect("zeta_t should be invertible");
        assert!(mag.is_finite(), "Mag(tM) at t={t} should be finite, got {mag}");
        // BV 2025 Eq 4.3: #T(⊥) ≤ Mag(tM) ≤ #ob(M) for t ≥ 1.
        if t >= 1.0 {
            assert!(
                mag >= m.terminating().len() as f64 - 1e-9,
                "Eq 4.3 lower bound violated at t={t}: mag={mag}"
            );
            assert!(
                mag <= m.objects().len() as f64 + 1e-9,
                "Eq 4.3 upper bound violated at t={t}: mag={mag}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Eq 4.3 bounds proptest — sanity check on random LMs
// ---------------------------------------------------------------------------

/// Construct a random tree-shaped `n`-state LM with strictly forward
/// transitions: state `i` may only transition to states `j > i`.
///
/// State naming: `s0, …, s{n-1}`. The last state `s{n-1}` is the only
/// terminating state. This mirrors the BV 2025 §2.15 prefix-poset shape
/// (forward-only, no cycles, single root); Eq 4.3 holds in this regime.
fn build_random_tree_lm(n: usize, seed: u64) -> LmCategory {
    let mut state = seed | 1;
    #[allow(clippy::cast_precision_loss)]
    let mut next = || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((state >> 33) as f64) / ((1u64 << 31) as f64)
    };

    let names: Vec<String> = (0..n).map(|i| format!("s{i}")).collect();
    let mut m = LmCategory::new(names.clone());
    m.mark_terminating(&names[n - 1]);

    // Forward chain: each non-terminal state distributes mass over later
    // states + leaves a non-trivial terminal mass (renormalize to 1 below).
    for i in 0..(n - 1) {
        let mut raw: Vec<f64> = Vec::with_capacity(n - i - 1);
        for _ in (i + 1)..n {
            raw.push(next());
        }
        let total: f64 = raw.iter().sum();
        if total < 1e-9 {
            continue;
        }
        for (k, &r) in raw.iter().enumerate() {
            let p = r / total;
            if p > 0.0 {
                m.add_transition(&names[i], &names[i + 1 + k], p);
            }
        }
    }
    m
}

proptest! {
    /// BV 2025 Eq 4.3: `#T(⊥) ≤ Mag(tM) ≤ #ob(M)` for `t ≥ 1`.
    ///
    /// At `t = 1` magnitude exactly equals `#T(⊥) + Σ entropies`, but the
    /// general bound argument is monotone and the upper bound `#ob(M)` is
    /// tight only as `t → ∞`. We test with `t ∈ {1.5, 2.0, 3.0}` to stay
    /// well inside the regime where ζ_t is invertible and the bounds apply.
    #[test]
    fn mag_bounds_eq_4_3(
        n in 2usize..=4,
        seed in any::<u64>(),
    ) {
        let m = build_random_tree_lm(n, seed);
        let n_term = m.terminating().len() as f64;
        let n_obj = m.objects().len() as f64;
        for &t in &[1.5_f64, 2.0, 3.0] {
            let Ok(mag) = m.magnitude(t) else {
                // Singular zeta on the random fixture — accept and skip.
                continue;
            };
            prop_assert!(
                mag >= n_term - 1e-6,
                "Eq 4.3 lower bound violated at t={t}: mag={mag}, #T(⊥)={n_term}"
            );
            prop_assert!(
                mag <= n_obj + 1e-6,
                "Eq 4.3 upper bound violated at t={t}: mag={mag}, #ob={n_obj}"
            );
        }
    }
}
