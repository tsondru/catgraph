//! Corelation: jointly-surjective cospan, composed by pushout.
//!
//! Dual of [`Rel`](crate::span::Rel); wraps [`Cospan`](crate::cospan::Cospan)
//! the way `Rel` wraps [`Span`](crate::span::Span).
//!
//! Realizes F&S 2018 (Seven Sketches) Example 6.64: Corel as a hypergraph category.

use std::fmt::Debug;

use crate::{
    cospan::Cospan,
    errors::CatgraphError,
};

/// A corelation: jointly-surjective cospan.
///
/// The dual of [`Rel`](crate::span::Rel). Composition is pushout composition
/// on the underlying cospan; this preserves joint surjectivity.
#[repr(transparent)]
pub struct Corel<Lambda: Eq + Sized + Debug + Copy>(Cospan<Lambda>);

impl<Lambda: Eq + Sized + Debug + Copy> Corel<Lambda> {
    /// Construct a corelation from a cospan, failing if the cospan is not jointly surjective.
    ///
    /// # Errors
    ///
    /// Returns [`CatgraphError::Corel`] if the cospan is not jointly surjective.
    pub fn new(cospan: Cospan<Lambda>) -> Result<Self, CatgraphError> {
        if !cospan.is_jointly_surjective() {
            return Err(CatgraphError::Corel {
                message: "cospan is not jointly surjective, cannot form a corelation".to_string(),
            });
        }
        Ok(Self(cospan))
    }

    /// Construct a corelation without checking joint surjectivity.
    /// Caller must guarantee the invariant.
    #[must_use]
    pub fn new_unchecked(cospan: Cospan<Lambda>) -> Self {
        Self(cospan)
    }

    /// View the underlying cospan (for bridge-crate access).
    #[must_use]
    pub fn as_cospan(&self) -> &Cospan<Lambda> {
        &self.0
    }
}

// Trait impls — all delegate to the underlying Cospan.

impl<Lambda> crate::category::HasIdentity<Vec<Lambda>> for Corel<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
{
    fn identity(on_this: &Vec<Lambda>) -> Self {
        // Cospan::identity on an n-element set is jointly surjective
        // (both legs are the identity map, hitting every middle vertex).
        Self(Cospan::<Lambda>::identity(on_this))
    }
}

impl<Lambda> crate::category::Composable<Vec<Lambda>> for Corel<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
{
    fn compose(&self, other: &Self) -> Result<Self, CatgraphError> {
        // Pushout composition of jointly-surjective cospans is jointly surjective.
        self.0.compose(&other.0).map(Self::new_unchecked)
    }

    fn domain(&self) -> Vec<Lambda> {
        self.0.domain()
    }

    fn codomain(&self) -> Vec<Lambda> {
        self.0.codomain()
    }

    fn composable(&self, other: &Self) -> Result<(), CatgraphError> {
        self.0.composable(&other.0)
    }
}

impl<Lambda> crate::monoidal::Monoidal for Corel<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
{
    fn monoidal(&mut self, other: Self) {
        // Disjoint union of jointly-surjective cospans is jointly surjective.
        self.0.monoidal(other.0);
    }
}

impl<Lambda> crate::monoidal::MonoidalMorphism<Vec<Lambda>> for Corel<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
{
}

impl<Lambda> crate::monoidal::SymmetricMonoidalMorphism<Lambda> for Corel<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
{
    fn from_permutation(
        p: permutations::Permutation,
        types: &[Lambda],
        types_as_on_domain: bool,
    ) -> Result<Self, CatgraphError> {
        // Cospan::from_permutation on an n-element set produces a jointly-surjective cospan.
        Cospan::<Lambda>::from_permutation(p, types, types_as_on_domain).map(Self::new_unchecked)
    }

    fn permute_side(&mut self, p: &permutations::Permutation, of_codomain: bool) {
        self.0.permute_side(p, of_codomain);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corel_new_accepts_jointly_surjective() {
        let c = Cospan::new(vec![0], vec![1], vec!['a', 'b']);
        let result = Corel::new(c);
        assert!(result.is_ok());
    }

    #[test]
    fn corel_new_rejects_non_surjective() {
        let c = Cospan::new(vec![0], vec![1], vec!['a', 'b', 'c']);
        let result = Corel::new(c);
        assert!(matches!(result, Err(CatgraphError::Corel { .. })));
    }

    #[test]
    fn corel_new_unchecked_bypasses_validation() {
        let c = Cospan::new(vec![0], vec![1], vec!['a', 'b', 'c']);
        let _corel = Corel::new_unchecked(c);
        // no panic, no error — invariant is caller's responsibility
    }

    #[test]
    fn corel_as_cospan_returns_underlying() {
        let c = Cospan::new(vec![0], vec![1], vec!['a', 'b']);
        let corel = Corel::new(c).unwrap();
        assert_eq!(corel.as_cospan().middle(), &['a', 'b']);
    }

    #[test]
    fn corel_identity_is_jointly_surjective() {
        use crate::category::HasIdentity;
        let types = vec!['a', 'b'];
        let id = Corel::<char>::identity(&types);
        assert!(id.as_cospan().is_jointly_surjective());
        assert_eq!(id.as_cospan().middle(), &['a', 'b']);
    }

    #[test]
    fn corel_compose_identity_left_is_noop() {
        use crate::category::{Composable, HasIdentity};
        let types = vec!['a'];
        let id = Corel::<char>::identity(&types);
        let composed = id.compose(&id).unwrap();
        assert!(composed.as_cospan().is_jointly_surjective());
        assert_eq!(composed.as_cospan().middle(), &['a']);
    }

    #[test]
    fn corel_domain_codomain_from_underlying_cospan() {
        use crate::category::Composable;
        let c = Cospan::new(vec![0], vec![1], vec!['a', 'b']);
        let corel = Corel::new(c).unwrap();
        assert_eq!(corel.domain(), vec!['a']);
        assert_eq!(corel.codomain(), vec!['b']);
    }

    #[test]
    fn corel_monoidal_preserves_surjectivity() {
        use crate::monoidal::Monoidal;
        let c1 = Cospan::new(vec![0], vec![0], vec!['a']);
        let c2 = Cospan::new(vec![0], vec![0], vec!['b']);
        let mut corel1 = Corel::new(c1).unwrap();
        let corel2 = Corel::new(c2).unwrap();
        corel1.monoidal(corel2);
        assert!(corel1.as_cospan().is_jointly_surjective());
    }
}
