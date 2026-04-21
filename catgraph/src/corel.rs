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
}
