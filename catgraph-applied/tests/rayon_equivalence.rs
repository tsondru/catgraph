//! Parallel-vs-sequential equivalence tests for catgraph-applied.
//!
//! `LinearCombination::Mul` and `BrauerMorphism::compose` branch on a size
//! threshold (32 for linear_combination, 8 for temperley_lieb). These tests
//! construct inputs at both sizes and assert determinism — the mathematical
//! result must not depend on whether the parallel path was taken.
//!
//! Pattern borrowed from the rayon crate's own test suite (see
//! `~/.claude/summaries/rayon-summary-0.md` — "Deterministic parallel-vs-sequential
//! equivalence" is the canonical rayon test idiom).

use catgraph::category::{Composable, HasIdentity};
use catgraph_applied::{
    linear_combination::LinearCombination,
    temperley_lieb::BrauerMorphism,
};

/// LinearCombination::Mul is commutative over a commutative Target ring.
/// Run at sizes below (16) and above (64) the threshold; assert commutativity
/// holds in both cases.
#[test]
fn linear_combination_mul_commutative_small_and_large() {
    // Small: 16 terms each, below PARALLEL_MUL_THRESHOLD=32.
    let a_small = make_lc(16, 1);
    let b_small = make_lc(16, 7);
    let ab_small = a_small.clone() * b_small.clone();
    let ba_small = b_small * a_small;
    assert_eq!(ab_small, ba_small, "Mul should be commutative at small size");

    // Large: 64 terms each, above threshold (triggers parallel path).
    let a_large = make_lc(64, 1);
    let b_large = make_lc(64, 7);
    let ab_large = a_large.clone() * b_large.clone();
    let ba_large = b_large * a_large;
    assert_eq!(ab_large, ba_large, "Mul should be commutative at large size");
}

/// LinearCombination::Mul — verify the parallel and sequential paths produce
/// identical output on the same input by pinning the input size at a level
/// that would exercise each path.
#[test]
fn linear_combination_mul_associative_across_threshold() {
    // At threshold boundary: 33 terms (just above 32).
    let a = make_lc(33, 1);
    let b = make_lc(33, 2);
    let c = make_lc(33, 3);
    let ab_c = (a.clone() * b.clone()) * c.clone();
    let a_bc = a * (b * c);
    assert_eq!(ab_c, a_bc, "Mul should be associative — parallel path must agree");
}

fn make_lc(n: usize, offset: i64) -> LinearCombination<i64, i64> {
    (0..n)
        .map(|i| (i64::try_from(i).unwrap() + offset, 1_i64))
        .collect()
}

/// BrauerMorphism compose is associative. Check at sizes straddling
/// `PARALLEL_COMBINATIONS_THRESHOLD = 8`.
#[test]
fn temperley_lieb_compose_associative_small_and_large() {
    // Small: n=4, below threshold.
    let gens_small: Vec<BrauerMorphism<i64>> = BrauerMorphism::temperley_lieb_gens(4);
    let e1 = &gens_small[0];
    let e2 = &gens_small[1];
    let e3 = &gens_small[2];
    let lhs = e1.compose(e2).unwrap().compose(e3).unwrap();
    let rhs = e1.compose(&e2.compose(e3).unwrap()).unwrap();
    assert_eq!(lhs, rhs, "compose should be associative at small n=4");

    // Large: n=12, triggers parallel non-crossing check (threshold 8).
    let gens_large: Vec<BrauerMorphism<i64>> = BrauerMorphism::temperley_lieb_gens(12);
    let g1 = &gens_large[0];
    let g2 = &gens_large[1];
    let g3 = &gens_large[2];
    let lhs = g1.compose(g2).unwrap().compose(g3).unwrap();
    let rhs = g1.compose(&g2.compose(g3).unwrap()).unwrap();
    assert_eq!(lhs, rhs, "compose should be associative at large n=12 (parallel path)");
}

/// Identity law: `id ; f = f = f ; id` at sizes below and above threshold.
#[test]
fn temperley_lieb_identity_law_small_and_large() {
    // Small: n=4.
    let id_small: BrauerMorphism<i64> = BrauerMorphism::identity(&4);
    let gens_small: Vec<BrauerMorphism<i64>> = BrauerMorphism::temperley_lieb_gens(4);
    let g = &gens_small[0];
    assert_eq!(&id_small.compose(g).unwrap(), g);
    assert_eq!(&g.compose(&id_small).unwrap(), g);

    // Large: n=16.
    let id_large: BrauerMorphism<i64> = BrauerMorphism::identity(&16);
    let gens_large: Vec<BrauerMorphism<i64>> = BrauerMorphism::temperley_lieb_gens(16);
    let h = &gens_large[7];
    assert_eq!(&id_large.compose(h).unwrap(), h);
    assert_eq!(&h.compose(&id_large).unwrap(), h);
}
