//! Tests for the v0.5.1 `NormalizeResult` struct semantics.

use catgraph_applied::prop::presentation::Presentation;
use catgraph_applied::prop::{Free, PropExpr, PropSignature};

#[derive(Clone, Debug, Eq, PartialEq)]
enum G {
    A,
    B,
}

impl PropSignature for G {
    fn source(&self) -> usize {
        1
    }
    fn target(&self) -> usize {
        1
    }
}

fn a() -> PropExpr<G> {
    Free::<G>::generator(G::A)
}
fn b() -> PropExpr<G> {
    Free::<G>::generator(G::B)
}

#[test]
fn normalize_result_converged_on_simple_reduction() {
    let mut p = Presentation::<G>::new();
    p.add_equation(a(), b()).unwrap();
    let result = p.normalize(&a()).unwrap();
    assert!(result.converged, "simple A→B rewrite should converge");
    assert_eq!(result.expr, b());
    assert!(result.steps_taken >= 1);
}

#[test]
fn normalize_result_hits_bound_on_cycle() {
    // A → A;A is a non-terminating expansion (the RHS always has a fresh
    // leftmost A subterm to rewrite). Must hit the depth bound.
    let mut p = Presentation::<G>::with_depth(4);
    let a_then_a = Free::<G>::compose(a(), a()).unwrap();
    p.add_equation(a(), a_then_a).unwrap();
    let result = p.normalize(&a()).unwrap();
    assert!(
        !result.converged,
        "A → A;A expansion at depth 4 should hit bound"
    );
    assert_eq!(result.steps_taken, 4);
}

#[test]
fn normalize_result_preserves_original_on_zero_equations() {
    let p = Presentation::<G>::new();
    let result = p.normalize(&a()).unwrap();
    assert!(result.converged);
    assert_eq!(result.expr, a());
    assert_eq!(result.steps_taken, 0);
}
