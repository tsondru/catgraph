//! Hypergraph functors: structure-preserving maps between hypergraph categories
//! (Fong-Spivak §2.3, Definition 2.12, Eq. 12).
//!
//! A **hypergraph functor** is a strong symmetric monoidal functor that preserves
//! Frobenius structure: `F(μ_X) = μ_{F(X)}`, and likewise for η, δ, ε.
//!
//! The [`HypergraphFunctor`] trait is generic over source and target categories,
//! each of which must implement [`HypergraphCategory`].
//!
//! ## Implementations
//!
//! - [`RelabelingFunctor`]: relabels `Cospan<L1> → Cospan<L2>` via a function `L1 → L2`,
//!   preserving all structural maps (the free hypergraph functor induced by a set map).
//!
//! ## Future implementations
//!
//! - The existing [`cospan_to_frobenius`](crate::cospan_algebra) function is a candidate
//!   second impl targeting `FrobeniusMorphism` as the target category, once
//!   `FrobeniusMorphism` implements `HypergraphCategory`.

use std::fmt::Debug;

use crate::{
    cospan::Cospan,
    errors::CatgraphError,
    hypergraph_category::HypergraphCategory,
};

/// A strong symmetric monoidal functor between hypergraph categories
/// that preserves Frobenius structure (Fong-Spivak §2.3, Eq. 12).
///
/// # Type parameters
///
/// - `L1`: label set of the source category
/// - `L2`: label set of the target category
/// - `Src`: source morphism type (must impl `HypergraphCategory<L1>`)
/// - `Tgt`: target morphism type (must impl `HypergraphCategory<L2>`)
///
/// # Laws (not enforced at compile time)
///
/// A correct implementation must satisfy:
///
/// **Frobenius preservation (Eq. 12):**
/// - `map_mor(Src::unit(x)) = Tgt::unit(map_ob(x))`
/// - `map_mor(Src::counit(x)) = Tgt::counit(map_ob(x))`
/// - `map_mor(Src::multiplication(x)) = Tgt::multiplication(map_ob(x))`
/// - `map_mor(Src::comultiplication(x)) = Tgt::comultiplication(map_ob(x))`
///
/// **Functoriality:**
/// - `map_mor(f ; g) = map_mor(f) ; map_mor(g)`
/// - `map_mor(id_x) = id_{map_ob(x)}`
///
/// **Monoidal:**
/// - `map_mor(f ⊗ g) = map_mor(f) ⊗ map_mor(g)`
pub trait HypergraphFunctor<L1, L2, Src, Tgt>
where
    L1: Eq + Copy + Debug,
    L2: Eq + Copy + Debug,
    Src: HypergraphCategory<L1>,
    Tgt: HypergraphCategory<L2>,
{
    /// Object mapping: sends each label in the source to a label in the target.
    fn map_ob(&self, x: L1) -> L2;

    /// Morphism mapping: sends a source morphism to a target morphism.
    ///
    /// Must be a strong symmetric monoidal functor preserving Frobenius structure.
    ///
    /// # Errors
    ///
    /// Returns [`CatgraphError`] if the mapping fails (e.g., incompatible structure).
    fn map_mor(&self, f: &Src) -> Result<Tgt, CatgraphError>;
}

/// A hypergraph functor induced by a label-relabeling function `L1 → L2`.
///
/// Implements `HypergraphFunctor<L1, L2, Cospan<L1>, Cospan<L2>>`. The structural
/// maps (left-to-middle, right-to-middle indices) are preserved unchanged; only the
/// middle set labels are transformed.
///
/// This is the free hypergraph functor `Cospan_f` induced by a set map `f: L1 → L2`
/// (Fong-Spivak §3.2).
pub struct RelabelingFunctor<F> {
    relabel: F,
}

impl<F> RelabelingFunctor<F> {
    /// Create a relabeling functor from a label-mapping function.
    #[must_use]
    pub fn new(relabel: F) -> Self {
        Self { relabel }
    }
}

impl<L1, L2, F> HypergraphFunctor<L1, L2, Cospan<L1>, Cospan<L2>>
    for RelabelingFunctor<F>
where
    L1: Eq + Copy + Debug,
    L2: Eq + Copy + Debug,
    F: Fn(L1) -> L2,
{
    fn map_ob(&self, x: L1) -> L2 {
        (self.relabel)(x)
    }

    fn map_mor(&self, f: &Cospan<L1>) -> Result<Cospan<L2>, CatgraphError> {
        let new_middle: Vec<L2> = f.middle().iter().map(|z| (self.relabel)(*z)).collect();
        Ok(Cospan::new(
            f.left_to_middle().to_vec(),
            f.right_to_middle().to_vec(),
            new_middle,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::{Composable, HasIdentity};

    fn char_to_u32(c: char) -> u32 {
        c as u32
    }

    #[test]
    fn relabeling_map_ob() {
        let f = RelabelingFunctor::new(char_to_u32);
        assert_eq!(f.map_ob('a'), 97);
        assert_eq!(f.map_ob('z'), 122);
    }

    #[test]
    fn relabeling_map_mor_identity() {
        let f = RelabelingFunctor::new(char_to_u32);
        let id = Cospan::<char>::identity(&vec!['a', 'b']);
        let mapped = f.map_mor(&id).unwrap();
        assert_eq!(mapped.domain(), vec![97, 98]);
        assert_eq!(mapped.codomain(), vec![97, 98]);
        assert!(mapped.is_left_identity());
        assert!(mapped.is_right_identity());
    }

    #[test]
    fn relabeling_map_mor_preserves_structure() {
        let f = RelabelingFunctor::new(char_to_u32);
        // merge cospan: [a, a] → [a], both left nodes to middle[0]
        let merge = Cospan::new(vec![0, 0], vec![0], vec!['a']);
        let mapped = f.map_mor(&merge).unwrap();
        assert_eq!(mapped.left_to_middle(), merge.left_to_middle());
        assert_eq!(mapped.right_to_middle(), merge.right_to_middle());
        assert_eq!(mapped.middle(), &[97]);
    }

    #[test]
    fn relabeling_empty_cospan() {
        let f = RelabelingFunctor::new(char_to_u32);
        let empty = Cospan::<char>::empty();
        let mapped = f.map_mor(&empty).unwrap();
        assert!(mapped.domain().is_empty());
        assert!(mapped.codomain().is_empty());
        assert!(mapped.middle().is_empty());
    }
}
