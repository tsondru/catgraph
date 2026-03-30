use std::marker::PhantomData;

use itertools::Itertools;

use crate::{
    errors::CatgraphError,
    frobenius::{Contains, InterpretableMorphism},
};

use {
    crate::category::{Composable, ComposableMutating, HasIdentity},
    std::fmt::Debug,
};

pub trait Monoidal {
    /*
    change the morphism self to the morphism (self \otimes other)
    */
    fn monoidal(&mut self, other: Self);
}

#[derive(PartialEq, Eq, Clone)]
pub struct GenericMonoidalMorphismLayer<BoxType, Lambda>
where
    Lambda: Eq + Copy,
    BoxType: Eq + Clone,
{
    /*
    a single layer for a black box filled morphism
    in a monoidal category whose objects
        are presented as tensor products of Lambda
    the black boxes are labelled with BoxType
    */
    pub blocks: Vec<BoxType>,
    pub left_type: Vec<Lambda>,
    pub right_type: Vec<Lambda>,
}

impl<Lambda, BoxType> Contains<BoxType> for GenericMonoidalMorphismLayer<BoxType, Lambda>
where
    Lambda: Eq + Copy,
    BoxType: Eq + Clone,
{
    fn contained_labels(&self) -> Vec<BoxType> {
        self.blocks.clone()
    }
}

impl<BoxType, Lambda> GenericMonoidalMorphismLayer<BoxType, Lambda>
where
    Lambda: Eq + Copy,
    BoxType: Eq + Clone,
{
    pub fn new() -> Self {
        Self {
            blocks: vec![],
            left_type: vec![],
            right_type: vec![],
        }
    }
}

impl<BoxType, Lambda> Default for GenericMonoidalMorphismLayer<BoxType, Lambda>
where
    Lambda: Eq + Copy,
    BoxType: Eq + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<BoxType, Lambda> HasIdentity<Vec<Lambda>> for GenericMonoidalMorphismLayer<BoxType, Lambda>
where
    Lambda: Eq + Copy,
    BoxType: Eq + Clone + HasIdentity<Lambda>,
{
    fn identity(on_type: &Vec<Lambda>) -> Self {
        let mut answer = Self::new();
        for cur_type in on_type {
            let op = BoxType::identity(cur_type);
            answer.blocks.push(op);
            answer.left_type.push(*cur_type);
            answer.right_type.push(*cur_type);
        }
        answer
    }
}

impl<BoxType, Lambda> Monoidal for GenericMonoidalMorphismLayer<BoxType, Lambda>
where
    Lambda: Eq + Copy,
    BoxType: Eq + Clone,
{
    fn monoidal(&mut self, other: Self) {
        self.blocks.extend(other.blocks);
        self.left_type.extend(other.left_type);
        self.right_type.extend(other.right_type);
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct GenericMonoidalMorphism<BoxType, Lambda>
where
    Lambda: Eq + Copy,
    BoxType: Eq + Clone,
{
    /*
    a black box filled morphism
    in a monoidal category whose objects
        are presented as tensor products of Lambda
    the black boxes are labelled with BoxType
    when given a function from BoxType to the
        actual type for the morphisms in the desired category
        one can interpret this as the aforementioned type
        by building up with composition and monoidal
    */
    layers: Vec<GenericMonoidalMorphismLayer<BoxType, Lambda>>,
}

impl<BoxType, Lambda> Default for GenericMonoidalMorphism<BoxType, Lambda>
where
    Lambda: Eq + Copy,
    BoxType: Eq + Clone,
{
    fn default() -> Self {
        Self { layers: vec![] }
    }
}

impl<Lambda, BoxType> Contains<BoxType> for GenericMonoidalMorphism<BoxType, Lambda>
where
    Lambda: Eq + Copy,
    BoxType: Eq + Clone,
{
    fn contained_labels(&self) -> Vec<BoxType> {
        #[allow(clippy::redundant_closure_for_method_calls)]
        self.layers
            .iter()
            .flat_map(|layer| layer.contained_labels())
            .collect_vec()
    }
}

impl<Lambda, BoxType> GenericMonoidalMorphism<BoxType, Lambda>
where
    Lambda: Eq + Copy,
    BoxType: Eq + Clone,
{
    pub fn new() -> Self {
        Self { layers: vec![] }
    }

    pub fn depth(&self) -> usize {
        self.layers.len()
    }

    #[allow(dead_code)]
    fn append_layer(
        &mut self,
        next_layer: GenericMonoidalMorphismLayer<BoxType, Lambda>,
    ) -> Result<(), CatgraphError> {
        let last_so_far = self.layers.pop();
        match last_so_far {
            None => {
                self.layers.push(next_layer);
            }
            Some(v) => {
                if v.right_type != next_layer.left_type {
                    return Err(CatgraphError::Composition("type mismatch in morphims composition".to_string()));
                }
                self.layers.push(v);
                self.layers.push(next_layer);
            }
        }
        Ok(())
    }

    pub fn extract_layers(self) -> Vec<GenericMonoidalMorphismLayer<BoxType, Lambda>> {
        self.layers
    }
}

impl<Lambda, BoxType> HasIdentity<Vec<Lambda>> for GenericMonoidalMorphism<BoxType, Lambda>
where
    Lambda: Eq + Copy,
    BoxType: Eq + Clone + HasIdentity<Lambda>,
{
    fn identity(on_this: &Vec<Lambda>) -> Self {
        Self {
            layers: vec![<_>::identity(on_this)],
        }
    }
}

impl<Lambda, BoxType> Monoidal for GenericMonoidalMorphism<BoxType, Lambda>
where
    Lambda: Eq + Copy + Debug,
    BoxType: Eq + Clone + HasIdentity<Lambda>,
{
    #[allow(clippy::assigning_clones)]
    fn monoidal(&mut self, other: Self) {
        let self_len = self.layers.len();
        let others_len = other.layers.len();
        let mut last_other_type: Vec<Lambda> = vec![];
        let mut last_self_type: Vec<Lambda> = vec![];
        for (n, cur_self_layer) in self.layers.iter_mut().enumerate() {
            last_self_type = cur_self_layer.right_type.clone();
            if n < other.layers.len() {
                last_other_type = other.layers[n].right_type.clone();
                cur_self_layer.monoidal(other.layers[n].clone());
            } else {
                cur_self_layer.monoidal(<_>::identity(&last_other_type));
            }
        }
        for n in self_len..others_len {
            let mut new_layer = GenericMonoidalMorphismLayer::identity(&last_self_type);
            new_layer.monoidal(other.layers[n].clone());
            let _ = self.append_layer(new_layer);
        }
    }
}

fn layers_composable<Lambda, BoxType>(
    l: &[GenericMonoidalMorphismLayer<BoxType, Lambda>],
    r: &[GenericMonoidalMorphismLayer<BoxType, Lambda>],
) -> Result<(), CatgraphError>
where
    Lambda: Eq + Copy + Debug,
    BoxType: Eq + Clone,
{
    if l.is_empty() || r.is_empty() {
        if l.is_empty() && r.is_empty() {
            return Ok(());
        }
        let interface = if l.is_empty() {
            &r[0].left_type
        } else {
            &l.last().unwrap().right_type
        };
        return if interface.is_empty() {
            Ok(())
        } else {
            Err(CatgraphError::Composition("Mismatch in cardinalities of common interface".to_string()))
        };
    }
    let self_interface = &l.last().unwrap().right_type;
    let other_interface = &r[0].left_type;
    if self_interface.len() != other_interface.len() {
        Err(CatgraphError::Composition("Mismatch in cardinalities of common interface".to_string()))
    } else if self_interface != other_interface {
        for idx in 0..self_interface.len() {
            let w1 = self_interface[idx];
            let w2 = other_interface[idx];
            if w1 != w2 {
                return Err(CatgraphError::Composition(format!(
                    "Mismatch in labels of common interface. At some index there was {w1:?} vs {w2:?}"
                )));
            }
        }
        Err(CatgraphError::Composition("Mismatch in labels of common interface at some unknown index.".to_string()))
    } else {
        Ok(())
    }
}

impl<Lambda, BoxType> ComposableMutating<Vec<Lambda>> for GenericMonoidalMorphism<BoxType, Lambda>
where
    Lambda: Eq + Copy + Debug,
    BoxType: Eq + Clone,
{
    fn composable(&self, other: &Self) -> Result<(), CatgraphError> {
        layers_composable(&self.layers, &other.layers)
    }

    fn compose(&mut self, other: Self) -> Result<(), CatgraphError> {
        for next_layer in other.layers {
            self.append_layer(next_layer)?;
        }
        Ok(())
    }

    fn domain(&self) -> Vec<Lambda> {
        self.layers
            .first()
            .map(|x| x.left_type.clone())
            .unwrap_or_default()
    }

    fn codomain(&self) -> Vec<Lambda> {
        self.layers
            .last()
            .map(|x| x.right_type.clone())
            .unwrap_or_default()
    }
}

#[allow(clippy::module_name_repetitions)]
pub trait MonoidalMorphism<T: Eq>: Monoidal + Composable<T> {}
#[allow(clippy::module_name_repetitions)]
pub trait MonoidalMutatingMorphism<T: Eq>: Monoidal + ComposableMutating<T> {}

impl<Lambda, BoxType> MonoidalMutatingMorphism<Vec<Lambda>>
    for GenericMonoidalMorphism<BoxType, Lambda>
where
    Lambda: Eq + Copy + Debug,
    BoxType: Eq + HasIdentity<Lambda> + Clone,
{
    /*
    the most obvious implementation of MonoidalMutatingMorphism is GenericMonoidalMorphism itself
    use all the structure of monoidal, compose, identity provided by concatenating blocks and layers appropriately
    */
}

struct InterpretableNoMut<T, Lambda>
where
    Lambda: Eq,
    T: Monoidal + Composable<Vec<Lambda>> + HasIdentity<Vec<Lambda>>,
{
    me: T,
    dummy: PhantomData<Lambda>,
}

impl<T, Lambda> InterpretableNoMut<T, Lambda>
where
    Lambda: Eq,
    T: Monoidal + Composable<Vec<Lambda>> + HasIdentity<Vec<Lambda>>,
{
    #[allow(dead_code)]
    fn change_black_boxer<F1, BoxType>(
        f1: F1,
    ) -> impl Fn(&BoxType, &[Lambda], &[Lambda]) -> Result<Self, CatgraphError>
    where
        F1: Fn(&BoxType) -> Result<T, CatgraphError>,
    {
        move |bb, _, _| f1(bb).map(Self::from)
    }
}

impl<T, Lambda> From<T> for InterpretableNoMut<T, Lambda>
where
    Lambda: Eq,
    T: Monoidal + Composable<Vec<Lambda>> + HasIdentity<Vec<Lambda>>,
{
    fn from(me: T) -> Self {
        Self {
            me,
            dummy: PhantomData,
        }
    }
}

struct InterpretableMut<T, Lambda>
where
    Lambda: Eq,
    T: Monoidal + ComposableMutating<Vec<Lambda>> + HasIdentity<Vec<Lambda>>,
{
    me: T,
    dummy: PhantomData<Lambda>,
}

impl<T, Lambda> InterpretableMut<T, Lambda>
where
    Lambda: Eq,
    T: Monoidal + ComposableMutating<Vec<Lambda>> + HasIdentity<Vec<Lambda>>,
{
    #[allow(dead_code)]
    fn change_black_boxer<F1, BoxType>(
        f1: F1,
    ) -> impl Fn(&BoxType, &[Lambda], &[Lambda]) -> Result<Self, CatgraphError>
    where
        F1: Fn(&BoxType) -> Result<T, CatgraphError>,
    {
        move |bb, _, _| f1(bb).map(Self::from)
    }
}

impl<T, Lambda> From<T> for InterpretableMut<T, Lambda>
where
    Lambda: Eq,
    T: Monoidal + ComposableMutating<Vec<Lambda>> + HasIdentity<Vec<Lambda>>,
{
    fn from(me: T) -> Self {
        Self {
            me,
            dummy: PhantomData,
        }
    }
}

impl<Lambda, BoxType, T>
    InterpretableMorphism<GenericMonoidalMorphism<BoxType, Lambda>, Lambda, BoxType>
    for InterpretableMut<T, Lambda>
where
    Lambda: Eq + Copy + Debug,
    BoxType: Eq + Clone,
    T: Monoidal + ComposableMutating<Vec<Lambda>> + HasIdentity<Vec<Lambda>>,
{
    /*
    given a function from BoxType to the
        actual type (Self) for the morphisms in the desired category
        one can interpret a GenericaMonoidalMorphism as a Self
        by building up with composition and monoidal
    */
    fn interpret<F>(
        morphism: &GenericMonoidalMorphism<BoxType, Lambda>,
        black_box_interpreter: F,
    ) -> Result<Self, CatgraphError>
    where
        F: Fn(&BoxType, &[Lambda], &[Lambda]) -> Result<Self, CatgraphError>,
    {
        let mut answer = T::identity(&morphism.domain());
        for layer in &morphism.layers {
            let Some(first) = &layer.blocks.first() else {
                return Err(CatgraphError::Interpret("somehow an empty layer in a generica monoidal morphism???".to_string()));
            };
            let mut cur_layer = black_box_interpreter(first, &[], &[]).map(|z| z.me)?;
            for block in &layer.blocks[1..] {
                cur_layer.monoidal(black_box_interpreter(block, &[], &[]).map(|z| z.me)?);
            }
            answer.compose(cur_layer)?;
        }
        Ok(Self::from(answer))
    }
}

impl<Lambda, BoxType, T>
    InterpretableMorphism<GenericMonoidalMorphism<BoxType, Lambda>, Lambda, BoxType>
    for InterpretableNoMut<T, Lambda>
where
    Lambda: Eq + Copy + Debug,
    BoxType: Eq + Clone,
    T: Monoidal + Composable<Vec<Lambda>> + HasIdentity<Vec<Lambda>>,
{
    /*
    given a function from BoxType to the
        actual type (Self) for the morphisms in the desired category
        one can interpret a GenericaMonoidalMorphism as a Self
        by building up with composition and monoidal
    only different from above because of the distinction between compositions
        that are done by modifying self to the composition self;other
        or that return a new self;other
    */
    fn interpret<F>(
        morphism: &GenericMonoidalMorphism<BoxType, Lambda>,
        black_box_interpreter: F,
    ) -> Result<Self, CatgraphError>
    where
        F: Fn(&BoxType, &[Lambda], &[Lambda]) -> Result<Self, CatgraphError>,
    {
        let mut answer = T::identity(&morphism.domain());
        for layer in &morphism.layers {
            let Some(first) = &layer.blocks.first() else {
                return Err(CatgraphError::Interpret("somehow an empty layer in a generica monoidal morphism???".to_string()));
            };
            let mut cur_layer = black_box_interpreter(first, &[], &[]).map(|z| z.me)?;
            for block in &layer.blocks[1..] {
                cur_layer.monoidal(black_box_interpreter(block, &[], &[]).map(|z| z.me)?);
            }
            answer = answer.compose(&cur_layer)?;
        }
        Ok(Self::from(answer))
    }
}

// ── Traits from symmetric_monoidal ──

use permutations::Permutation;

#[allow(clippy::module_name_repetitions)]
pub trait SymmetricMonoidalMorphism<T: Eq> {
    /*
    can pre/post compose a given morphism with a permutation (possibly panic if the permutation is not of the right cardinality)
    give the morphism : types[0] otimes \cdots -> types[p[0]] \otimes \cdots
        or the inverse (depending on types_as_on_domain)
        again can panic if the cardinality of the permutation does not match the cardinality of types
    */
    fn permute_side(&mut self, p: &Permutation, of_codomain: bool);
    fn from_permutation(p: Permutation, types: &[T], types_as_on_domain: bool) -> Self;
}

#[allow(clippy::module_name_repetitions)]
pub trait SymmetricMonoidalDiscreteMorphism<T: Eq> {
    /*
    for finset they are morphisms on finite sets, but rather than specify the domain/codomain as Vec<Singleton>
    the domain and codomain are just treated as usize, so we can't use the above trait where types was a slice
    */
    fn permute_side(&mut self, p: &Permutation, of_codomain: bool);
    fn from_permutation(p: Permutation, types: T, types_as_on_domain: bool) -> Self;
}

#[cfg(test)]
mod test {
    use super::*;

    // Simple test type for boxes
    #[derive(Clone, PartialEq, Eq, Debug)]
    struct SimpleBox {
        input: Vec<char>,
        output: Vec<char>,
    }

    impl HasIdentity<char> for SimpleBox {
        fn identity(on_this: &char) -> Self {
            SimpleBox {
                input: vec![*on_this],
                output: vec![*on_this],
            }
        }
    }

    #[test]
    fn layer_new() {
        let layer: GenericMonoidalMorphismLayer<SimpleBox, char> =
            GenericMonoidalMorphismLayer::new();
        assert!(layer.blocks.is_empty());
        assert!(layer.left_type.is_empty());
        assert!(layer.right_type.is_empty());
    }

    #[test]
    fn layer_identity() {
        let types = vec!['a', 'b', 'c'];
        let layer: GenericMonoidalMorphismLayer<SimpleBox, char> =
            GenericMonoidalMorphismLayer::identity(&types);

        assert_eq!(layer.blocks.len(), 3);
        assert_eq!(layer.left_type, types);
        assert_eq!(layer.right_type, types);
    }

    #[test]
    fn layer_monoidal() {
        let layer1: GenericMonoidalMorphismLayer<SimpleBox, char> =
            GenericMonoidalMorphismLayer::identity(&vec!['a']);
        let layer2: GenericMonoidalMorphismLayer<SimpleBox, char> =
            GenericMonoidalMorphismLayer::identity(&vec!['b']);

        let mut combined = layer1;
        combined.monoidal(layer2);

        assert_eq!(combined.blocks.len(), 2);
        assert_eq!(combined.left_type, vec!['a', 'b']);
        assert_eq!(combined.right_type, vec!['a', 'b']);
    }

    #[test]
    fn layer_contained_labels() {
        let types = vec!['a', 'b'];
        let layer: GenericMonoidalMorphismLayer<SimpleBox, char> =
            GenericMonoidalMorphismLayer::identity(&types);

        let labels = layer.contained_labels();
        assert_eq!(labels.len(), 2);
    }

    #[test]
    fn morphism_new() {
        let morphism: GenericMonoidalMorphism<SimpleBox, char> = GenericMonoidalMorphism::new();
        assert_eq!(morphism.depth(), 0);
    }

    #[test]
    fn morphism_identity() {
        let types = vec!['a', 'b'];
        let morphism: GenericMonoidalMorphism<SimpleBox, char> =
            GenericMonoidalMorphism::identity(&types);

        assert_eq!(morphism.depth(), 1);
        assert_eq!(morphism.domain(), types);
        assert_eq!(morphism.codomain(), types);
    }

    #[test]
    fn morphism_monoidal() {
        let m1: GenericMonoidalMorphism<SimpleBox, char> =
            GenericMonoidalMorphism::identity(&vec!['a']);
        let m2: GenericMonoidalMorphism<SimpleBox, char> =
            GenericMonoidalMorphism::identity(&vec!['b']);

        let mut combined = m1;
        combined.monoidal(m2);

        assert_eq!(combined.domain(), vec!['a', 'b']);
        assert_eq!(combined.codomain(), vec!['a', 'b']);
    }

    #[test]
    fn morphism_compose() {
        let types = vec!['a', 'b'];
        let m1: GenericMonoidalMorphism<SimpleBox, char> = GenericMonoidalMorphism::identity(&types);
        let m2: GenericMonoidalMorphism<SimpleBox, char> = GenericMonoidalMorphism::identity(&types);

        let mut composed = m1;
        let result = composed.compose(m2);
        assert!(result.is_ok());
        assert_eq!(composed.depth(), 2);
    }

    #[test]
    fn morphism_compose_mismatch() {
        let m1: GenericMonoidalMorphism<SimpleBox, char> =
            GenericMonoidalMorphism::identity(&vec!['a']);
        let m2: GenericMonoidalMorphism<SimpleBox, char> =
            GenericMonoidalMorphism::identity(&vec!['b']);

        let mut composed = m1;
        let result = composed.compose(m2);
        assert!(result.is_err());
    }

    #[test]
    fn morphism_composable() {
        let types = vec!['a'];
        let m1: GenericMonoidalMorphism<SimpleBox, char> = GenericMonoidalMorphism::identity(&types);
        let m2: GenericMonoidalMorphism<SimpleBox, char> = GenericMonoidalMorphism::identity(&types);

        assert!(m1.composable(&m2).is_ok());

        let m3: GenericMonoidalMorphism<SimpleBox, char> =
            GenericMonoidalMorphism::identity(&vec!['b']);
        assert!(m1.composable(&m3).is_err());
    }

    #[test]
    fn morphism_extract_layers() {
        let types = vec!['a', 'b'];
        let morphism: GenericMonoidalMorphism<SimpleBox, char> =
            GenericMonoidalMorphism::identity(&types);

        let layers = morphism.extract_layers();
        assert_eq!(layers.len(), 1);
    }

    #[test]
    fn morphism_contained_labels() {
        let types = vec!['a', 'b'];
        let morphism: GenericMonoidalMorphism<SimpleBox, char> =
            GenericMonoidalMorphism::identity(&types);

        let labels = morphism.contained_labels();
        assert_eq!(labels.len(), 2);
    }

    #[test]
    fn morphism_append_layer() {
        let types = vec!['a'];
        let mut morphism: GenericMonoidalMorphism<SimpleBox, char> =
            GenericMonoidalMorphism::identity(&types);

        let layer: GenericMonoidalMorphismLayer<SimpleBox, char> =
            GenericMonoidalMorphismLayer::identity(&types);

        let result = morphism.append_layer(layer);
        assert!(result.is_ok());
        assert_eq!(morphism.depth(), 2);
    }

    #[test]
    fn morphism_append_layer_empty() {
        let types = vec!['a'];
        let mut morphism: GenericMonoidalMorphism<SimpleBox, char> = GenericMonoidalMorphism::new();

        let layer: GenericMonoidalMorphismLayer<SimpleBox, char> =
            GenericMonoidalMorphismLayer::identity(&types);

        let result = morphism.append_layer(layer);
        assert!(result.is_ok());
        assert_eq!(morphism.depth(), 1);
    }

    #[test]
    fn layers_composable_both_empty() {
        let l: Vec<GenericMonoidalMorphismLayer<SimpleBox, char>> = vec![];
        let r: Vec<GenericMonoidalMorphismLayer<SimpleBox, char>> = vec![];
        assert!(layers_composable(&l, &r).is_ok());
    }

    #[test]
    fn layers_composable_one_empty() {
        let types = vec!['a'];
        let layer: GenericMonoidalMorphismLayer<SimpleBox, char> =
            GenericMonoidalMorphismLayer::identity(&types);
        let l = vec![layer];
        let r: Vec<GenericMonoidalMorphismLayer<SimpleBox, char>> = vec![];

        // Non-empty interface with empty other side should fail
        assert!(layers_composable(&l, &r).is_err());
    }

    #[test]
    fn monoidal_different_depths() {
        // Test monoidal with morphisms of different depths
        let mut m1: GenericMonoidalMorphism<SimpleBox, char> =
            GenericMonoidalMorphism::identity(&vec!['a']);
        let mut m2: GenericMonoidalMorphism<SimpleBox, char> =
            GenericMonoidalMorphism::identity(&vec!['b']);

        // Make m2 deeper by composing with itself
        let m2_copy: GenericMonoidalMorphism<SimpleBox, char> =
            GenericMonoidalMorphism::identity(&vec!['b']);
        let _ = m2.compose(m2_copy);

        m1.monoidal(m2);
        assert_eq!(m1.domain(), vec!['a', 'b']);
    }
}
