//! Integration tests for `Presentation<G>` and the SMC-axiom term rewriter.

use catgraph::errors::CatgraphError;
use catgraph_applied::prop::{presentation::Presentation, Free, PropExpr, PropSignature};

// ---- Tiny signature for testing ----
//
// Three generators A, B, C, all arity 1→1, encoded directly as enum variants
// (the signature trait requires the generator type itself to implement
// `PropSignature` with its own `source()` / `target()` methods).

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum TestGen {
    A,
    B,
    C,
}

impl PropSignature for TestGen {
    fn source(&self) -> usize {
        1
    }
    fn target(&self) -> usize {
        1
    }
}

fn g(x: TestGen) -> PropExpr<TestGen> {
    Free::<TestGen>::generator(x)
}

// ---- Tests ----

#[test]
fn empty_presentation_applies_smc_rules_only() {
    // (Identity(0) ⊗ A).normalize  should reduce to just A (left unitor).
    let pres = Presentation::<TestGen>::new();
    let expr = Free::<TestGen>::tensor(Free::<TestGen>::identity(0), g(TestGen::A));
    let normalized = pres.normalize(&expr).unwrap().expr;
    assert_eq!(normalized, g(TestGen::A));
}

#[test]
fn user_equation_applied_left_to_right() {
    // Presentation with A = B.
    let mut pres = Presentation::<TestGen>::new();
    pres.add_equation(g(TestGen::A), g(TestGen::B)).unwrap();
    let normalized = pres.normalize(&g(TestGen::A)).unwrap().expr;
    assert_eq!(normalized, g(TestGen::B));
}

#[test]
fn eq_mod_detects_smc_interchange() {
    // (A ⊗ B) ; (A ⊗ B)  vs  (A ; A) ⊗ (B ; B) — should be SMC-equal via interchange.
    let pres = Presentation::<TestGen>::new();

    let lhs = Free::<TestGen>::compose(
        Free::<TestGen>::tensor(g(TestGen::A), g(TestGen::B)),
        Free::<TestGen>::tensor(g(TestGen::A), g(TestGen::B)),
    )
    .unwrap();

    let rhs = Free::<TestGen>::tensor(
        Free::<TestGen>::compose(g(TestGen::A), g(TestGen::A)).unwrap(),
        Free::<TestGen>::compose(g(TestGen::B), g(TestGen::B)).unwrap(),
    );

    assert!(
        pres.eq_mod(&lhs, &rhs).unwrap().unwrap_or(false),
        "(A⊗B);(A⊗B) should SMC-equal (A;A)⊗(B;B)"
    );
}

#[test]
fn arity_mismatch_on_add_equation_rejected() {
    // A is 1→1; (A ⊗ A) is 2→2. Can't equate them.
    let mut pres = Presentation::<TestGen>::new();
    let a_tensor_a = Free::<TestGen>::tensor(g(TestGen::A), g(TestGen::A));
    let result = pres.add_equation(g(TestGen::A), a_tensor_a);
    assert!(matches!(result, Err(CatgraphError::Presentation { .. })));
}

#[test]
fn depth_bound_respected_on_cyclic_rewrite() {
    // Cyclic rewrite A → B, B → A. Normalize must terminate within depth bound.
    let mut pres = Presentation::<TestGen>::with_depth(16);
    pres.add_equation(g(TestGen::A), g(TestGen::B)).unwrap();
    pres.add_equation(g(TestGen::B), g(TestGen::A)).unwrap();

    // Must return *some* PropExpr within bound, even if not unique.
    let result = pres.normalize(&g(TestGen::A));
    assert!(
        result.is_ok(),
        "normalize must terminate under cyclic rewrites: {result:?}"
    );
}

#[test]
fn braid_involution_smc_rule() {
    // Braid(1,1) ; Braid(1,1) should normalize to Identity(2).
    let pres = Presentation::<TestGen>::new();
    let expr =
        Free::<TestGen>::compose(Free::<TestGen>::braid(1, 1), Free::<TestGen>::braid(1, 1))
            .unwrap();
    let normalized = pres.normalize(&expr).unwrap().expr;
    assert_eq!(normalized, Free::<TestGen>::identity(2));
}

#[test]
fn identity_unitor_right_smc_rule() {
    // A ⊗ Identity(0) should normalize to A.
    let pres = Presentation::<TestGen>::new();
    let expr = Free::<TestGen>::tensor(g(TestGen::A), Free::<TestGen>::identity(0));
    let normalized = pres.normalize(&expr).unwrap().expr;
    assert_eq!(normalized, g(TestGen::A));
}

#[test]
fn compose_identity_reduction_smc_rule() {
    // Identity(1) ; A should normalize to A.
    let pres = Presentation::<TestGen>::new();
    let expr = Free::<TestGen>::compose(Free::<TestGen>::identity(1), g(TestGen::A)).unwrap();
    let normalized = pres.normalize(&expr).unwrap().expr;
    assert_eq!(normalized, g(TestGen::A));
}

// Unused in passing runs; keeps the import honest if a later assertion needs
// it. Present to mirror the spec's triad TestGen::{A, B, C}.
#[allow(dead_code)]
fn _use_c() -> PropExpr<TestGen> {
    g(TestGen::C)
}
