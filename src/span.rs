use crate::errors::CatgraphError;

use {
    crate::{
        category::{Composable, HasIdentity},
        monoidal::{Monoidal, MonoidalMorphism},
        monoidal::SymmetricMonoidalMorphism,
        utils::{in_place_permute, represents_id},
    },
    either::Either::{self, Left, Right},
    std::{collections::HashSet, fmt::Debug},
};

type LeftIndex = usize;
type RightIndex = usize;
type MiddleIndex = usize;

#[derive(Clone)]
pub struct Span<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
{
    middle: Vec<(LeftIndex, RightIndex)>, // the leg maps from the source to the domain and codomain sets
    left: Vec<Lambda>,                    // the labels on the domain
    right: Vec<Lambda>,                   // the labels on the codomain
    is_left_id: bool,
    is_right_id: bool,
}

impl<Lambda> Span<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
{
    pub fn assert_valid(&self, check_id_strong: bool, check_id_weak: bool) {
        let left_size = self.left.len();
        let left_in_bounds = self.middle.iter().all(|(z, _)| *z < left_size);
        assert!(
            left_in_bounds,
            "A target for one of the left arrows was out of bounds"
        );
        let right_size = self.right.len();
        let right_in_bounds = self.middle.iter().all(|(_, z)| *z < right_size);
        assert!(
            right_in_bounds,
            "A target for one of the right arrows was out of bounds"
        );
        let left_right_types_match = self
            .middle
            .iter()
            .all(|(z1, z2)| self.left[*z1] == self.right[*z2]);
        assert!(
            left_right_types_match,
            "There was a left and right linked by something in the span, but their lambda types didn't match"
        );
        if check_id_strong || (check_id_weak && self.is_left_id) {
            let is_left_really_id = represents_id(self.middle_to_left().into_iter());
            assert_eq!(
                is_left_really_id, self.is_left_id,
                "The identity nature of the left arrow was wrong"
            );
        }
        if check_id_strong || (check_id_weak && self.is_right_id) {
            let is_right_really_id = represents_id(self.middle_to_right().into_iter());
            assert_eq!(
                is_right_really_id, self.is_right_id,
                "The identity nature of the right arrow was wrong"
            );
        }
    }

    pub fn new(
        left: Vec<Lambda>,
        right: Vec<Lambda>,
        middle: Vec<(LeftIndex, RightIndex)>,
    ) -> Self {
        let is_left_id = represents_id(middle.iter().map(|tup| tup.0));
        let is_right_id = represents_id(middle.iter().map(|tup| tup.1));
        let answer = Self {
            middle,
            left,
            right,
            is_left_id,
            is_right_id,
        };
        answer.assert_valid(false, false);
        answer
    }

    pub fn left(&self) -> &[Lambda] {
        &self.left
    }

    pub fn right(&self) -> &[Lambda] {
        &self.right
    }

    pub fn middle_pairs(&self) -> &[(LeftIndex, RightIndex)] {
        &self.middle
    }

    pub fn is_left_identity(&self) -> bool {
        self.is_left_id
    }

    pub fn is_right_identity(&self) -> bool {
        self.is_right_id
    }

    pub fn middle_to_left(&self) -> Vec<LeftIndex> {
        self.middle.iter().map(|tup| tup.0).collect()
    }

    pub fn middle_to_right(&self) -> Vec<RightIndex> {
        self.middle.iter().map(|tup| tup.1).collect()
    }

    pub fn add_boundary_node(
        &mut self,
        new_boundary: Either<Lambda, Lambda>,
    ) -> Either<LeftIndex, RightIndex> {
        /*
        add a new boundary node with specified label
        which side depends on whether Left or Right
        it is not in the image of the leg maps
        but the index is returned so the caller
            can put it in the image of the leg maps later
        */
        match new_boundary {
            Left(z) => {
                self.left.push(z);
                Left(self.left.len() - 1)
            }
            Right(z) => {
                self.right.push(z);
                Right(self.right.len() - 1)
            }
        }
    }

    pub fn add_middle(
        &mut self,
        new_middle: (LeftIndex, RightIndex),
    ) -> Result<MiddleIndex, String> {
        /*
        add a new node at the source
        the leg maps on this new node send it to the specified indices
        the labels of these two targets must match up
        upon success, the index of this new node is returned
        */
        let type_left = self.left[new_middle.0];
        let type_right = self.right[new_middle.1];
        if type_left != type_right {
            return Err(format!(
                "Mismatched lambda values {type_left:?} and {type_right:?}"
            ));
        }
        self.middle.push(new_middle);
        self.is_left_id = false;
        self.is_right_id = false;
        Ok(self.middle.len() - 1)
    }

    pub fn map<F, Mu>(&self, f: F) -> Span<Mu>
    where
        F: Fn(Lambda) -> Mu,
        Mu: Sized + Eq + Copy + Debug,
    {
        /*
        change the labels with the function f
        */
        Span::new(
            self.left.iter().map(|l| f(*l)).collect(),
            self.right.iter().map(|l| f(*l)).collect(),
            self.middle.clone(),
        )
    }

    pub fn is_jointly_injective(&self) -> bool {
        /*
        could this span be interpreted as a Relation instead of just a span
        if the leg maps are jointly injective, then might as well say the source node
        is a subset of the domain \times codomain, instead of it's own new set
        */
        crate::utils::is_unique(&self.middle)
    }

    pub fn dagger(&self) -> Self {
        /*
        flip the domain and codomain
        */
        Self::new(
            self.codomain(),
            self.domain(),
            self.middle.iter().map(|(z, w)| (*w, *z)).collect(),
        )
    }
}

impl<Lambda> HasIdentity<Vec<Lambda>> for Span<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
{
    fn identity(on_this: &Vec<Lambda>) -> Self {
        Self {
            middle: (0..on_this.len()).map(|idx| (idx, idx)).collect(),
            left: on_this.clone(),
            right: on_this.clone(),
            is_left_id: true,
            is_right_id: true,
        }
    }
}

impl<Lambda> Composable<Vec<Lambda>> for Span<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
{
    fn composable(&self, other: &Self) -> Result<(), CatgraphError> {
        crate::utils::same_labels_check(self.right.iter(), other.left.iter())
            .map_err(CatgraphError::Composition)
    }

    fn compose(&self, other: &Self) -> Result<Self, CatgraphError> {
        self.composable(other)?;
        // could shortuct if self.is_right_id or other.is_left_id, but unnecessary
        let max_middle = self.middle.len().max(other.middle.len());
        let mut answer = Self::new(
            self.left.clone(),
            other.right.clone(),
            Vec::with_capacity(max_middle),
        );
        for (sl, sr) in &self.middle {
            for (ol, or) in &other.middle {
                if sr == ol {
                    let mid_added = answer.add_middle((*sl, *or));
                    match mid_added {
                        Ok(_) => {}
                        Err(z) => {
                            return Err(CatgraphError::Composition(format!("{z}\nShould be unreachable if composability already said it was all okay.")));
                        }
                    }
                }
            }
        }
        Ok(answer)
    }

    fn domain(&self) -> Vec<Lambda> {
        self.left.clone()
    }

    fn codomain(&self) -> Vec<Lambda> {
        self.right.clone()
    }
}

impl<Lambda> Monoidal for Span<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
{
    fn monoidal(&mut self, mut other: Self) {
        self.is_left_id &= other.is_left_id;
        self.is_right_id &= other.is_right_id;
        let left_shift = self.left.len();
        let right_shift = self.right.len();
        other.middle.iter_mut().for_each(|(v1, v2)| {
            *v1 += left_shift;
            *v2 += right_shift;
        });
        self.middle.extend(other.middle);
        self.left.extend(other.left);
        self.right.extend(other.right);
    }
}

impl<Lambda> MonoidalMorphism<Vec<Lambda>> for Span<Lambda> where Lambda: Sized + Eq + Copy + Debug {}

impl<Lambda> SymmetricMonoidalMorphism<Lambda> for Span<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
{
    fn permute_side(&mut self, p: &permutations::Permutation, of_codomain: bool) {
        if of_codomain {
            self.is_right_id = false;
            in_place_permute(&mut self.right, p);
            self.middle.iter_mut().for_each(|(_, v2)| {
                *v2 = p.apply(*v2);
            });
        } else {
            self.is_left_id = false;
            in_place_permute(&mut self.left, p);
            self.middle.iter_mut().for_each(|(v1, _)| {
                *v1 = p.apply(*v1);
            });
        }
        //todo test
    }

    fn from_permutation(
        p: permutations::Permutation,
        types: &[Lambda],
        types_as_on_domain: bool,
    ) -> Self {
        if types_as_on_domain {
            Self {
                left: types.to_vec(),
                middle: (0..types.len()).map(|idx| (idx, p.apply(idx))).collect(),
                right: p.inv().permute(types),
                is_left_id: true,
                is_right_id: false,
            }
        } else {
            Self {
                left: p.inv().permute(types),
                middle: (0..types.len()).map(|idx| (p.apply(idx), idx)).collect(),
                right: types.to_vec(),
                is_left_id: false,
                is_right_id: true,
            }
        }
    }
}

/*
wrapper around span for rel
where the source is now always assumed to be a subset of the product
by the leg maps being jointly injective
*/
#[repr(transparent)]
pub struct Rel<Lambda: Eq + Sized + Debug + Copy>(Span<Lambda>);

impl<Lambda> HasIdentity<Vec<Lambda>> for Rel<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
{
    fn identity(on_this: &Vec<Lambda>) -> Self {
        Self(Span::<Lambda>::identity(on_this))
    }
}

impl<Lambda> Composable<Vec<Lambda>> for Rel<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
{
    fn compose(&self, other: &Self) -> Result<Self, CatgraphError> {
        self.0.compose(&other.0).map(|x| Self(x))
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

impl<Lambda> Monoidal for Rel<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
{
    fn monoidal(&mut self, other: Self) {
        self.0.monoidal(other.0);
    }
}

impl<Lambda> MonoidalMorphism<Vec<Lambda>> for Rel<Lambda> where Lambda: Sized + Eq + Copy + Debug {}

impl<Lambda: Eq + Sized + Debug + Copy> Rel<Lambda> {
    pub fn as_span(&self) -> &Span<Lambda> {
        &self.0
    }

    pub fn new(x: Span<Lambda>) -> Result<Self, CatgraphError> {
        if !x.is_jointly_injective() {
            return Err(CatgraphError::Relation(
                "span is not jointly injective, cannot form a relation".to_string(),
            ));
        }
        Ok(Self(x))
    }

    pub fn new_unchecked(x: Span<Lambda>) -> Self {
        Self(x)
    }

    /// # Panics
    /// Panics if `self` and `other` have different domain or codomain.
    pub fn subsumes(&self, other: &Rel<Lambda>) -> bool {
        assert_eq!(self.domain(), other.domain());
        assert_eq!(self.codomain(), other.codomain());

        #[allow(clippy::from_iter_instead_of_collect)]
        let self_pairs: HashSet<(usize, usize)> = HashSet::from_iter(self.0.middle.iter().copied());
        #[allow(clippy::from_iter_instead_of_collect)]
        let other_pairs: HashSet<(usize, usize)> =
            HashSet::from_iter(other.0.middle.iter().copied());

        self_pairs.is_superset(&other_pairs)
    }

    /// # Panics
    /// Panics if `self` and `other` have different domain or codomain.
    pub fn union(&self, other: &Self) -> Self {
        assert_eq!(self.domain(), other.domain());
        assert_eq!(self.codomain(), other.codomain());

        #[allow(clippy::from_iter_instead_of_collect)]
        let self_pairs: HashSet<(usize, usize)> = HashSet::from_iter(self.0.middle.iter().copied());
        let mut ret_val = self.0.clone();
        for (x, y) in &other.0.middle {
            if !self_pairs.contains(&(*x, *y)) {
                /*
                the reason this would panic is if the labels mismatched
                but because the labels for the Left(x) and Right(y) nodes
                matched when other was created, we know they do
                so the unwrap should never panic
                */
                ret_val.add_middle((*x, *y)).unwrap();
            }
        }
        Self(ret_val)
    }

    /// # Panics
    /// Panics if `self` and `other` have different domain or codomain.
    pub fn intersection(&self, other: &Self) -> Self {
        assert_eq!(self.domain(), other.domain());
        assert_eq!(self.codomain(), other.codomain());

        let capacity = self.0.middle.len().min(other.0.middle.len());
        let mut ret_val =
            Span::<Lambda>::new(self.domain(), self.codomain(), Vec::with_capacity(capacity));

        #[allow(clippy::from_iter_instead_of_collect)]
        let self_pairs: HashSet<(usize, usize)> = HashSet::from_iter(self.0.middle.iter().copied());
        #[allow(clippy::from_iter_instead_of_collect)]
        let other_pairs: HashSet<(usize, usize)> =
            HashSet::from_iter(other.0.middle.iter().copied());

        let in_common = self_pairs.intersection(&other_pairs);
        for (x, y) in in_common {
            ret_val.add_middle((*x, *y)).unwrap();
        }
        Self(ret_val)
    }

    pub fn complement(&self) -> Result<Self, CatgraphError> {
        /*
        say self is a relation with domain and codomain A and B
        make a new relation with (A \times B) \setminus self
        there can be errors if there are label mismatches
        */
        let source_size = self.domain().len();
        let target_size = self.codomain().len();

        let capacity = source_size * target_size - self.0.middle.len();
        let mut ret_val =
            Span::<Lambda>::new(self.domain(), self.codomain(), Vec::with_capacity(capacity));

        #[allow(clippy::from_iter_instead_of_collect)]
        let self_pairs: HashSet<(usize, usize)> = HashSet::from_iter(self.0.middle.iter().copied());

        for x in 0..source_size {
            for y in 0..target_size {
                if !self_pairs.contains(&(x, y)) {
                    ret_val
                        .add_middle((x, y))
                        .map_err(CatgraphError::Relation)?;
                }
            }
        }
        Ok(Self(ret_val))
    }

    pub fn is_homogeneous(&self) -> bool {
        /*
        a relation with the same domain and codomain
        */
        self.0.domain() == self.0.codomain()
    }

    /// # Panics
    /// Panics if the relation is not homogeneous (domain != codomain).
    pub fn is_reflexive(&self) -> bool {
        let identity_rel = Self::new_unchecked(Span::<Lambda>::identity(&self.0.domain()));
        self.subsumes(&identity_rel)
    }

    pub fn is_irreflexive(&self) -> bool {
        /*
        is the complement reflexive
        if there are label mismatches when trying to create the complement, then return false
        */
        self.complement().map(|x| x.is_reflexive()).unwrap_or(false)
    }

    /// # Panics
    /// Panics if the relation is not homogeneous (domain != codomain).
    pub fn is_symmetric(&self) -> bool {
        let dagger = Self::new_unchecked(self.0.dagger());
        self.subsumes(&dagger)
    }

    /// # Panics
    /// Panics if the relation is not homogeneous (domain != codomain).
    pub fn is_antisymmetric(&self) -> bool {
        let dagger = Self::new_unchecked(self.0.dagger());
        let intersect = self.intersection(&dagger);
        let identity_rel = Self::new_unchecked(Span::<Lambda>::identity(&self.0.domain()));
        identity_rel.subsumes(&intersect)
    }

    /// # Panics
    /// Panics if the relation is not homogeneous (domain != codomain).
    pub fn is_transitive(&self) -> bool {
        // compose can't fail: homogeneous relation has matching domain/codomain
        let twice = Self::new_unchecked(self.0.compose(&self.0).unwrap());
        self.subsumes(&twice)
    }

    pub fn is_equivalence_rel(&self) -> bool {
        /*
        is this an equivalence relation
        so we can interpret a pair (x,y) being in the source of this relation as x \equiv y
        and not equivalent otherwise
        */
        self.is_homogeneous() && self.is_reflexive() && self.is_symmetric() && self.is_transitive()
    }

    pub fn is_partial_order(&self) -> bool {
        /*
        is this a partial order
        so we can interpret a pair (x,y) being in the source of this relation as x \leq y
        and not \leq otherwise
        */
        self.is_homogeneous()
            && self.is_reflexive()
            && self.is_antisymmetric()
            && self.is_transitive()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::category::{Composable, HasIdentity};
    use crate::monoidal::Monoidal;

    #[test]
    fn span_new_and_accessors() {
        // Create a simple span with matching types
        let left = vec!['a', 'b'];
        let right = vec!['a', 'b'];
        let middle = vec![(0, 0), (1, 1)];
        let span = Span::new(left.clone(), right.clone(), middle);

        assert_eq!(span.domain(), left);
        assert_eq!(span.codomain(), right);
        assert_eq!(span.middle_to_left(), vec![0, 1]);
        assert_eq!(span.middle_to_right(), vec![0, 1]);
    }

    #[test]
    fn span_identity() {
        let types = vec!['x', 'y', 'z'];
        let id = Span::identity(&types);

        assert_eq!(id.domain(), types);
        assert_eq!(id.codomain(), types);
        assert_eq!(id.middle_to_left(), vec![0, 1, 2]);
        assert_eq!(id.middle_to_right(), vec![0, 1, 2]);
        assert!(id.is_jointly_injective());
    }

    #[test]
    fn span_compose_identity() {
        let types = vec!['a', 'b'];
        let id = Span::identity(&types);
        let result = id.compose(&id);
        assert!(result.is_ok());
        let composed = result.unwrap();
        assert_eq!(composed.domain(), types);
        assert_eq!(composed.codomain(), types);
    }

    #[test]
    fn span_compose_general() {
        // f: {0,1} -> {a,b} x {a,b} where f maps to (0,0) and (1,1)
        // g: {0,1} -> {a,b} x {a,b} where g maps to (0,0) and (1,1)
        // f;g should have middle elements where f's right matches g's left
        let left = vec!['a', 'b'];
        let right = vec!['a', 'b'];
        let f = Span::new(left.clone(), right.clone(), vec![(0, 0), (1, 1)]);
        let g = Span::new(left.clone(), right.clone(), vec![(0, 0), (1, 1)]);

        let result = f.compose(&g);
        assert!(result.is_ok());
    }

    #[test]
    fn span_compose_mismatch() {
        // Spans with matching internal types but incompatible interfaces
        let span1 = Span::new(vec!['a'], vec!['a'], vec![(0, 0)]);
        let span2 = Span::new(vec!['b'], vec!['b'], vec![(0, 0)]);

        let result = span1.compose(&span2);
        assert!(result.is_err());
    }

    #[test]
    fn span_monoidal() {
        let span1 = Span::new(vec!['a'], vec!['a'], vec![(0, 0)]);
        let span2 = Span::new(vec!['b'], vec!['b'], vec![(0, 0)]);

        let mut combined = span1;
        combined.monoidal(span2);

        assert_eq!(combined.domain(), vec!['a', 'b']);
        assert_eq!(combined.codomain(), vec!['a', 'b']);
    }

    #[test]
    fn span_dagger() {
        // middle pairs must have matching types at their positions
        let span = Span::new(vec!['a', 'b'], vec!['a', 'b'], vec![(0, 0), (1, 1)]);
        let dagger = span.dagger();

        assert_eq!(dagger.domain(), vec!['a', 'b']);
        assert_eq!(dagger.codomain(), vec!['a', 'b']);
    }

    #[test]
    fn span_is_jointly_injective() {
        // Injective: no duplicate pairs
        let span1 = Span::new(vec!['a', 'b'], vec!['a', 'b'], vec![(0, 0), (1, 1)]);
        assert!(span1.is_jointly_injective());

        // Not injective: duplicate pair
        let span2 = Span::new(vec!['a', 'b'], vec!['a', 'b'], vec![(0, 0), (0, 0)]);
        assert!(!span2.is_jointly_injective());
    }

    #[test]
    fn span_add_middle() {
        let mut span = Span::new(vec!['a', 'b'], vec!['a', 'b'], vec![]);

        // Add matching types
        let result = span.add_middle((0, 0));
        assert!(result.is_ok());

        // Add mismatched types
        let result = span.add_middle((0, 1));
        assert!(result.is_err());
    }

    #[test]
    fn span_map() {
        let span = Span::new(vec![1, 2], vec![1, 2], vec![(0, 0), (1, 1)]);
        let mapped = span.map(|x| x * 10);

        assert_eq!(mapped.domain(), vec![10, 20]);
        assert_eq!(mapped.codomain(), vec![10, 20]);
    }

    #[test]
    fn span_add_boundary_node() {
        // Start with matching types
        let mut span = Span::new(vec!['a'], vec!['a'], vec![(0, 0)]);

        let left_idx = span.add_boundary_node(Left('c'));
        assert!(matches!(left_idx, Left(1)));

        let right_idx = span.add_boundary_node(Right('d'));
        assert!(matches!(right_idx, Right(1)));
    }

    // Rel tests
    #[test]
    fn rel_identity() {
        let types = vec!['a', 'b', 'c'];
        let id = Rel::identity(&types);

        assert_eq!(id.domain(), types);
        assert_eq!(id.codomain(), types);
    }

    #[test]
    fn rel_compose() {
        let types = vec!['a', 'b'];
        let id = Rel::identity(&types);

        let result = id.compose(&id);
        assert!(result.is_ok());
    }

    #[test]
    fn rel_monoidal() {
        let rel1 = Rel::identity(&vec!['a']);
        let rel2 = Rel::identity(&vec!['b']);

        let mut combined = rel1;
        combined.monoidal(rel2);

        assert_eq!(combined.domain(), vec!['a', 'b']);
    }

    #[test]
    fn rel_subsumes() {
        let types = vec!['a', 'b'];
        let full = Rel::new(
            Span::new(types.clone(), types.clone(), vec![(0, 0), (1, 1)]),
        ).unwrap();
        let partial = Rel::new(Span::new(types.clone(), types.clone(), vec![(0, 0)])).unwrap();

        assert!(full.subsumes(&partial));
        assert!(!partial.subsumes(&full));
    }

    #[test]
    fn rel_intersection() {
        let types = vec!['a', 'b'];
        let rel1 = Rel::new(
            Span::new(types.clone(), types.clone(), vec![(0, 0), (1, 1)]),
        ).unwrap();
        let rel2 = Rel::new(Span::new(types.clone(), types.clone(), vec![(0, 0)])).unwrap();

        let intersect = rel1.intersection(&rel2);
        assert_eq!(intersect.0.middle.len(), 1);
    }

    #[test]
    fn rel_union() {
        let types = vec!['a', 'b'];
        let rel1 = Rel::new(Span::new(types.clone(), types.clone(), vec![(0, 0)])).unwrap();
        let rel2 = Rel::new(Span::new(types.clone(), types.clone(), vec![(1, 1)])).unwrap();

        let union = rel1.union(&rel2);
        assert_eq!(union.0.middle.len(), 2);
    }

    #[test]
    fn rel_is_homogeneous() {
        let same = Rel::new(Span::new(vec!['a'], vec!['a'], vec![(0, 0)])).unwrap();
        assert!(same.is_homogeneous());

        // For non-homogeneous, use empty middle to avoid type mismatch validation
        let diff = Rel::new(Span::new(vec!['a'], vec!['b'], vec![])).unwrap();
        assert!(!diff.is_homogeneous());
    }

    #[test]
    fn rel_is_reflexive() {
        let types = vec!['a', 'b'];
        let reflexive = Rel::identity(&types);
        assert!(reflexive.is_reflexive());

        // For not reflexive, we can use a relation that's missing the diagonal
        // Use same type at all positions
        let same_types = vec!['a', 'a'];
        let not_reflexive =
            Rel::new(Span::new(same_types.clone(), same_types.clone(), vec![(0, 1)])).unwrap();
        assert!(!not_reflexive.is_reflexive());
    }

    #[test]
    fn rel_is_symmetric() {
        let types = vec!['a', 'a'];
        // Symmetric: contains both (0,1) and (1,0)
        let symmetric = Rel::new(
            Span::new(types.clone(), types.clone(), vec![(0, 1), (1, 0)]),
        ).unwrap();
        assert!(symmetric.is_symmetric());

        // Not symmetric: only (0,1)
        let not_symmetric =
            Rel::new(Span::new(types.clone(), types.clone(), vec![(0, 1)])).unwrap();
        assert!(!not_symmetric.is_symmetric());
    }

    #[test]
    fn rel_is_antisymmetric() {
        let types = vec!['a', 'a'];
        // Antisymmetric: identity relation
        let antisymmetric = Rel::identity(&types);
        assert!(antisymmetric.is_antisymmetric());
    }

    #[test]
    fn rel_is_transitive() {
        let types = vec!['a', 'a', 'a'];
        // Identity is transitive
        let transitive = Rel::identity(&types);
        assert!(transitive.is_transitive());
    }

    #[test]
    fn rel_is_equivalence() {
        let types = vec!['a', 'a'];
        // Identity is an equivalence relation
        let equiv = Rel::identity(&types);
        assert!(equiv.is_equivalence_rel());
    }

    #[test]
    fn rel_is_partial_order() {
        let types = vec!['a', 'a'];
        // Identity is a partial order
        let po = Rel::identity(&types);
        assert!(po.is_partial_order());
    }

    #[test]
    fn rel_complement_non_square() {
        // Non-square: 3-element domain, 2-element codomain
        // All labels 'a' so any (x, y) pair passes the type-match check
        let domain = vec!['a', 'a', 'a'];
        let codomain = vec!['a', 'a'];
        let pairs = vec![(0, 0), (2, 1)];
        let original_count = pairs.len();

        let rel = Rel::new(
            Span::new(domain.clone(), codomain.clone(), pairs),
        ).unwrap();

        let comp = rel.complement().expect("complement should succeed");

        // Full Cartesian product has 3*2 = 6 pairs
        let expected_complement_size = 3 * 2 - original_count;
        assert_eq!(
            comp.0.middle.len(),
            expected_complement_size,
            "complement of non-square relation should have source_size*target_size - original_count pairs"
        );

        // Verify specific pairs: complement should contain exactly the 4 pairs NOT in the original
        let comp_pairs: HashSet<(usize, usize)> =
            HashSet::from_iter(comp.0.middle.iter().copied());
        assert!(!comp_pairs.contains(&(0, 0)), "(0,0) was in original");
        assert!(!comp_pairs.contains(&(2, 1)), "(2,1) was in original");
        assert!(comp_pairs.contains(&(0, 1)));
        assert!(comp_pairs.contains(&(1, 0)));
        assert!(comp_pairs.contains(&(1, 1)));
        assert!(comp_pairs.contains(&(2, 0)));

        // Involution property: complement(complement(r)) == r
        let double_comp = comp.complement().expect("double complement should succeed");
        let original_pairs: HashSet<(usize, usize)> =
            HashSet::from_iter(rel.0.middle.iter().copied());
        let roundtrip_pairs: HashSet<(usize, usize)> =
            HashSet::from_iter(double_comp.0.middle.iter().copied());
        assert_eq!(
            original_pairs, roundtrip_pairs,
            "complement(complement(r)) should equal r"
        );
    }

    #[test]
    fn span_from_permutation_identity_domain() {
        use crate::monoidal::SymmetricMonoidalMorphism;
        use permutations::Permutation;

        let types = vec!['a', 'b', 'c'];
        let id_perm = Permutation::identity(3);
        let span = Span::from_permutation(id_perm, &types, true);

        assert_eq!(span.domain(), types);
        assert_eq!(span.codomain(), types);
        // Identity permutation: middle should map each index to itself
        assert_eq!(span.middle_to_left(), vec![0, 1, 2]);
        assert_eq!(span.middle_to_right(), vec![0, 1, 2]);
    }

    #[test]
    fn span_from_permutation_identity_codomain() {
        use crate::monoidal::SymmetricMonoidalMorphism;
        use permutations::Permutation;

        let types = vec!['x', 'y'];
        let id_perm = Permutation::identity(2);
        let span = Span::from_permutation(id_perm, &types, false);

        assert_eq!(span.domain(), types);
        assert_eq!(span.codomain(), types);
        assert_eq!(span.middle_to_left(), vec![0, 1]);
        assert_eq!(span.middle_to_right(), vec![0, 1]);
    }

    #[test]
    fn span_from_permutation_rotation_domain() {
        use crate::monoidal::SymmetricMonoidalMorphism;
        use permutations::Permutation;

        let types = vec!['a', 'b', 'c'];
        let rot = Permutation::rotation_left(3, 1);
        let span = Span::from_permutation(rot, &types, true);

        // types_as_on_domain=true: left=types, right=permuted types
        assert_eq!(span.domain(), vec!['a', 'b', 'c']);
        // rotation_left(3,1) sends 0->1, 1->2, 2->0
        // permute(types) reorders types by where each index maps
        assert_eq!(span.codomain().len(), 3);
        // middle maps each idx to rot.apply(idx)
        for (left_idx, right_idx) in span.middle.iter().copied() {
            // Types must match across the middle
            assert_eq!(span.left[left_idx], span.right[right_idx]);
        }
    }

    #[test]
    fn span_from_permutation_rotation_codomain() {
        use crate::monoidal::SymmetricMonoidalMorphism;
        use permutations::Permutation;

        let types = vec!['a', 'b', 'c'];
        let rot = Permutation::rotation_left(3, 1);
        let span = Span::from_permutation(rot, &types, false);

        // types_as_on_domain=false: right=types, left=permuted types
        assert_eq!(span.codomain(), vec!['a', 'b', 'c']);
        assert_eq!(span.domain().len(), 3);
        // middle maps (rot.apply(idx), idx)
        for (left_idx, right_idx) in span.middle.iter().copied() {
            assert_eq!(span.left[left_idx], span.right[right_idx]);
        }
    }
}
