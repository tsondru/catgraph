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
