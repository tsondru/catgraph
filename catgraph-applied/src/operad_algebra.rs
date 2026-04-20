//! Algebras over an operad.
//!
//! F&S *Seven Sketches* §6.5 **Def 6.99.** An *algebra* for an operad `O` is
//! a functor `F : O → Set`. Concretely, `F` sends each type of `O` to a
//! carrier set `F(X)` and each `n`-ary operation `o ∈ O(X_1, …, X_n; Y)` to
//! a function `F(o) : F(X_1) × … × F(X_n) → F(Y)` such that substitution in
//! `O` corresponds to composition of functions, and identities in `O` map
//! to identity functions on carriers.
//!
//! # This implementation
//!
//! The [`OperadAlgebra`] trait captures the single-sorted case: one carrier
//! set per operad (the associated type [`OperadAlgebra::Element`]) and a
//! uniform [`evaluate`](OperadAlgebra::evaluate) method that interprets
//! each operation of `O` as a function `Elementⁿ → Element`. Multi-sorted
//! (typed) operads are a v0.5.0 refinement.
//!
//! The trait is parameterised over the operad type `O` and the input-label
//! type `Input` so that the same algebra notion applies to all concrete
//! operads defined in this crate ([`E1`](crate::e1_operad::E1),
//! [`E2`](crate::e2_operad::E2),
//! [`WiringDiagram`](crate::wiring_diagram::WiringDiagram)).
//!
//! # Ex 6.100 worked example
//!
//! [`CircAlgebra`] implements the textbook's named example
//! `Circ : Cospan → Set` specialised to
//! [`WiringDiagram`](crate::wiring_diagram::WiringDiagram). See the example
//! `examples/operad_algebra_circ.rs` for a substitution-preservation demo.

use catgraph::errors::CatgraphError;
use catgraph::operadic::Operadic;

/// A single-sorted algebra `F : O → Set` for an operad `O`.
pub trait OperadAlgebra<O, Input>
where
    O: Operadic<Input>,
{
    /// Carrier set `F(X)` — one element type shared across all types of `O`.
    type Element: Clone;

    /// Interpret an operation `op` of arity `n` as a function
    /// `Elementⁿ → Element`.
    ///
    /// # Errors
    ///
    /// Returns [`CatgraphError`] when the caller-supplied `inputs` do not
    /// match the operation's declared arity or when the algebra cannot
    /// evaluate the operation for a domain-specific reason.
    fn evaluate(
        &self,
        op: &O,
        inputs: &[Self::Element],
    ) -> Result<Self::Element, CatgraphError>;
}
