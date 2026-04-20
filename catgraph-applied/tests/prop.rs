//! Tests for `catgraph_applied::prop` — F&S *Seven Sketches* §5.2 Def 5.2
//! (props) and Def 5.25 (free prop on a signature).

use catgraph_applied::prop::{Free, PropExpr, PropSignature};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum Sig {
    Mul,
    Unit,
}

impl PropSignature for Sig {
    fn source(&self) -> usize {
        match self {
            Sig::Mul => 2,
            Sig::Unit => 0,
        }
    }
    fn target(&self) -> usize {
        match self {
            Sig::Mul | Sig::Unit => 1,
        }
    }
}

#[test]
fn generator_carries_declared_arity() {
    let g: PropExpr<Sig> = Free::generator(Sig::Mul);
    assert_eq!(g.source(), 2);
    assert_eq!(g.target(), 1);
}

#[test]
fn identity_has_matching_source_and_target() {
    let i: PropExpr<Sig> = Free::identity(3);
    assert_eq!(i.source(), 3);
    assert_eq!(i.target(), 3);
}

#[test]
fn braid_has_sum_arity() {
    let b: PropExpr<Sig> = Free::braid(2, 3);
    assert_eq!(b.source(), 5);
    assert_eq!(b.target(), 5);
}

#[test]
fn compose_rejects_arity_mismatch() {
    let f: PropExpr<Sig> = Free::generator(Sig::Mul); // 2 -> 1
    let g: PropExpr<Sig> = Free::generator(Sig::Mul); // 2 -> 1
    // f.target() == 1 but g.source() == 2 -> must fail
    assert!(Free::compose(f, g).is_err());
}

#[test]
fn compose_accepts_matching_arities() {
    let f: PropExpr<Sig> = Free::generator(Sig::Mul); // 2 -> 1
    let id: PropExpr<Sig> = Free::identity(1); // 1 -> 1
    let r = Free::compose(f, id).expect("arities match");
    assert_eq!(r.source(), 2);
    assert_eq!(r.target(), 1);
}

#[test]
fn tensor_sums_arities() {
    let f: PropExpr<Sig> = Free::generator(Sig::Mul); // 2 -> 1
    let u: PropExpr<Sig> = Free::generator(Sig::Unit); // 0 -> 1
    let r = Free::tensor(f, u);
    assert_eq!(r.source(), 2);
    assert_eq!(r.target(), 2);
}
