//! Decorated cospans: cospans of finite sets equipped with extra structure on
//! the apex, following Fong & Spivak's *Seven Sketches in Compositionality*
//! (arXiv:1803.05316v3), Definition 6.75.
//!
//! A decorated cospan is a pair `(c, d)` where `c` is a cospan of finite sets
//! `X → N ← Y` and `d ∈ F(N)` is a decoration on the apex. The decoration
//! function is modelled by a lax symmetric monoidal functor
//! `F : (FinSet, +) → (Set, ×)`, i.e. a [`Decoration`] implementation.
//!
//! Under this framework, composition of decorated cospans uses the pushout of
//! the underlying cospans together with `F`'s pushforward on the coequalizer
//! quotient, and monoidal product uses the laxator `φ_{a,b}` to combine
//! decorations across disjoint apices. This is the bridge between the strict
//! cospan machinery in [`catgraph::cospan`] and domain-specific decorated
//! structures (open Petri nets, open graphs, open dynamical systems, …).
//!
//! ## What lives here (Task 3)
//!
//! This module currently provides only the bare skeleton:
//!
//! - the [`Decoration`] trait (F on objects + laxator + pushforward), and
//! - the generic [`DecoratedCospan`] struct with a simple constructor.
//!
//! ## Forthcoming (Tasks 4–5)
//!
//! The category-theoretic structure (`Composable`, `Monoidal`,
//! `HypergraphCategory` instances for `DecoratedCospan<Lambda, D>`) is
//! deferred to subsequent tasks. The target result is Fong–Spivak
//! **Theorem 6.77**: decorated cospans form a hypergraph category, with
//! special Frobenius structure inherited from the underlying `Cospan`.

use std::fmt::Debug;

use catgraph::cospan::Cospan;

/// A lax symmetric monoidal functor `F : (FinSet, +) → (Set, ×)` supplying
/// decorations on cospan apices.
///
/// Implementers specify what extra structure lives on top of the apex of a
/// cospan (graph edges, Petri net transitions, dynamical system laws, …) and
/// how that structure transforms under
///
/// 1. the empty apex (`F` on the initial object `0 ∈ FinSet`),
/// 2. disjoint union of apices (`F`'s laxator `φ_{a,b} : F(a) × F(b) → F(a+b)`),
///    and
/// 3. pushout quotients of the apex (`F` applied to the coequalizer map
///    produced during cospan composition).
///
/// Together these are exactly the data required to turn a span of cospans
/// into a decorated-cospan category (Fong–Spivak Def 6.75, Thm 6.77).
pub trait Decoration: Sized {
    /// The set `F(N)` of decorations on an apex of size `n`.
    type Apex: Clone + Debug + PartialEq;

    /// `F` on objects: the canonical "empty" decoration for an apex of size
    /// `n`. In most concrete instances this is a zero element, an empty edge
    /// set, or the unique element of a singleton. The parameter `n` is the
    /// apex cardinality and is retained because some decorations (e.g.
    /// vector-valued markings) depend on it even in the empty case.
    fn empty(n: usize) -> Self::Apex;

    /// `F` on `+`: combine decorations on disjoint apices into a decoration
    /// on their sum. Corresponds to the functor's laxator
    /// `φ_{a,b} : F(a) × F(b) → F(a + b)`.
    fn combine(a: Self::Apex, b: Self::Apex) -> Self::Apex;

    /// `F` on pushout quotients: given a decoration on the pre-pushout apex
    /// and the quotient map `q : {0, …, n-1} → {0, …, m-1}` (as a slice
    /// whose `i`th entry is the image of the `i`th pre-pushout element),
    /// produce the decoration on the pushed-out apex.
    ///
    /// This is the image under `F` of the coequalizer arrow that appears in
    /// cospan composition.
    fn pushforward(d: Self::Apex, quotient: &[usize]) -> Self::Apex;
}

/// A cospan of finite sets together with a decoration on its apex.
///
/// The `Lambda` parameter is the middle-vertex label type of the underlying
/// [`Cospan`]; the `D` parameter is a [`Decoration`] functor whose associated
/// apex type determines the shape of the decoration.
///
/// Note: `PartialEq` is intentionally **not** derived here, because the
/// upstream `Cospan<Lambda>` does not implement `PartialEq` (its identity
/// flags are cached and can make structurally equal cospans compare unequal).
/// Downstream code should compare the `cospan` fields through the public
/// leg/middle accessors and the `decoration` fields via their own `PartialEq`.
#[derive(Clone, Debug)]
pub struct DecoratedCospan<Lambda, D>
where
    Lambda: Eq + Copy + Debug,
    D: Decoration,
{
    /// The underlying (undecorated) cospan.
    pub cospan: Cospan<Lambda>,
    /// The decoration on the cospan's apex, valued in `F(|middle|)`.
    pub decoration: D::Apex,
}

impl<Lambda, D> DecoratedCospan<Lambda, D>
where
    Lambda: Eq + Copy + Debug,
    D: Decoration,
{
    /// Construct a decorated cospan from an underlying cospan and a decoration.
    ///
    /// No consistency check is performed between `cospan.middle().len()` and
    /// the shape of `decoration` — that invariant is the responsibility of
    /// the specific [`Decoration`] implementation.
    #[must_use]
    pub fn new(cospan: Cospan<Lambda>, decoration: D::Apex) -> Self {
        Self { cospan, decoration }
    }
}

#[cfg(test)]
mod tests {
    // The trivial decoration uses `type Apex = ()`, so every call to
    // `Trivial::{empty, combine, pushforward}` returns `()`. Clippy's
    // `let_unit_value` / `unit_arg` pedantic lints fire on every such call.
    // These lints are signal on real code but pure noise for a unit-valued
    // test double, so suppress them in this module only.
    #![allow(clippy::let_unit_value, clippy::unit_arg)]

    use super::{Decoration, DecoratedCospan};
    use catgraph::cospan::Cospan;

    /// The trivial decoration functor `F(n) = {*}`. Every apex carries the
    /// unique unit decoration; laxator and pushforward are forced.
    #[derive(Debug)]
    struct Trivial;

    impl Decoration for Trivial {
        type Apex = ();

        fn empty(_n: usize) -> Self::Apex {}

        fn combine(_a: Self::Apex, _b: Self::Apex) -> Self::Apex {}

        fn pushforward(_d: Self::Apex, _quotient: &[usize]) -> Self::Apex {}
    }

    #[test]
    fn trivial_decoration_sanity() {
        // Build a small char-labelled cospan: left=[0], right=[1], middle=['a','b'].
        let cospan = Cospan::<char>::new(vec![0], vec![1], vec!['a', 'b']);
        // `Trivial::empty(2)` returns `()`; bind explicitly so clippy's
        // `unit_arg` lint sees an intentional unit decoration rather than a
        // function call whose only return is `()`.
        let decoration: <Trivial as Decoration>::Apex = Trivial::empty(2);
        let decorated: DecoratedCospan<char, Trivial> =
            DecoratedCospan::new(cospan, decoration);

        assert_eq!(decorated.decoration, ());
        assert_eq!(decorated.cospan.middle(), &['a', 'b']);
        assert_eq!(decorated.cospan.left_to_middle(), &[0]);
        assert_eq!(decorated.cospan.right_to_middle(), &[1]);

        // Exercise the remaining `Decoration` methods so they aren't flagged
        // as dead code and so the sanity test covers the full trait surface.
        let combined: <Trivial as Decoration>::Apex =
            Trivial::combine(Trivial::empty(1), Trivial::empty(1));
        assert_eq!(combined, ());
        let pushed: <Trivial as Decoration>::Apex =
            Trivial::pushforward(Trivial::empty(2), &[0, 0]);
        assert_eq!(pushed, ());
    }
}
