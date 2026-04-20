//! Tests for `catgraph_applied::operad_algebra` — F&S *Seven Sketches*
//! §6.5 Def 6.99 (an algebra for an operad `O` is a functor `F : O → Set`).

use catgraph::category::HasIdentity;
use catgraph::errors::CatgraphError;
use catgraph_applied::e1_operad::E1;
use catgraph_applied::operad_algebra::OperadAlgebra;

/// Trivial algebra: every operation maps to the identity function on `()`.
/// Used only as a smoke test of the trait shape.
struct IdAlgebra;

impl OperadAlgebra<E1, usize> for IdAlgebra {
    type Element = ();
    fn evaluate(
        &self,
        _op: &E1,
        _inputs: &[Self::Element],
    ) -> Result<Self::Element, CatgraphError> {
        Ok(())
    }
}

#[test]
fn identity_algebra_evaluates() {
    let op = E1::identity(&());
    let r = IdAlgebra.evaluate(&op, &[()]).unwrap();
    assert_eq!(r, ());
}
