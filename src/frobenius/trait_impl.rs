use std::fmt::Debug;

use crate::errors::CatgraphError;

use {
    crate::{
        category::{ComposableMutating, HasIdentity},
        monoidal::{MonoidalMutatingMorphism, SymmetricMonoidalMorphism},
    },
    permutations::Permutation,
};

use super::{
    morphism_system::InterpretableMorphism,
    operations::{
        FrobeniusMorphism, FrobeniusOperation, special_frobenius_morphism,
    },
};

pub trait Frobenius<Lambda: Eq + Copy + Debug + Send + Sync, BlackBoxLabel: Eq + Clone + Send + Sync>:
    SymmetricMonoidalMorphism<Lambda> + HasIdentity<Vec<Lambda>> + MonoidalMutatingMorphism<Vec<Lambda>>
{
    /*
    the implementor (Self) of this trait is a type for a morphism in a symmetric monoidal category with
    objects built as tensor products of basic objects labelled from Lambda
    and each such basic object is a frobenius object with interpretations
    so one can interpret each of unit/counit/multiplication/comultiplication as a Self
    */
    fn interpret_unit(z: Lambda) -> Self;
    fn interpret_counit(z: Lambda) -> Self;
    fn interpret_multiplication(z: Lambda) -> Self;
    fn interpret_comultiplication(z: Lambda) -> Self;

    fn basic_interpret<F>(
        single_step: &FrobeniusOperation<Lambda, BlackBoxLabel>,
        black_box_interpreter: &F,
    ) -> Result<Self, CatgraphError>
    where
        F: Fn(&BlackBoxLabel, &[Lambda], &[Lambda]) -> Result<Self, CatgraphError>,
    {
        /*
        interpret a single frobenius operation as a Self
        with black_box_interpreter saying how to interpret the black boxes
            the black boxes do not have to be morphisms that can be built from Frobenius operations (though they might)
        the identity and symmetric braiding are interpreted
            using the fact that Self was a morphism in a symmetric monoidal category
        */
        Ok(match single_step {
            FrobeniusOperation::Unit(z) => Self::interpret_unit(*z),
            FrobeniusOperation::Counit(z) => Self::interpret_counit(*z),
            FrobeniusOperation::Multiplication(z) => Self::interpret_multiplication(*z),
            FrobeniusOperation::Comultiplication(z) => Self::interpret_comultiplication(*z),
            FrobeniusOperation::Identity(z) => Self::identity(&vec![*z]),
            FrobeniusOperation::SymmetricBraiding(z1, z2) => {
                let transposition = Permutation::try_from(vec![0, 1]).unwrap();
                Self::from_permutation(transposition, &[*z1, *z2], true)?
            }
            FrobeniusOperation::UnSpecifiedBox(bbl, z1, z2) => black_box_interpreter(bbl, z1, z2)?,
            FrobeniusOperation::Spider(z, d1, d2) => {
                let broken_down = special_frobenius_morphism(*d1, *d2, *z);
                Self::interpret_frob(&broken_down, black_box_interpreter)?
            }
        })
    }

    fn interpret_frob<F>(
        morphism: &FrobeniusMorphism<Lambda, BlackBoxLabel>,
        black_box_interpreter: &F,
    ) -> Result<Self, CatgraphError>
    where
        F: Fn(&BlackBoxLabel, &[Lambda], &[Lambda]) -> Result<Self, CatgraphError>,
    {
        /*
        interpret a complicated frobenius morphism as a Self
        built up from all the basic_interpret using composition and monoidal
        */
        let mut answer = Self::identity(&morphism.domain());
        for layer in &morphism.layers {
            if layer.blocks.is_empty() {
                return Err(CatgraphError::Interpret { context: "somehow an empty layer in a frobenius morphism???".to_string() });
            }
            let first = &layer.blocks[0];
            let mut cur_layer = Self::basic_interpret(&first.op, black_box_interpreter)?;
            for block in &layer.blocks[1..] {
                cur_layer.monoidal(Self::basic_interpret(&block.op, black_box_interpreter)?);
            }
            answer.compose(cur_layer)?;
        }
        Ok(answer)
    }
}

impl<Lambda, BlackBoxLabel> Frobenius<Lambda, BlackBoxLabel>
    for FrobeniusMorphism<Lambda, BlackBoxLabel>
where
    Lambda: Eq + Copy + Debug + Send + Sync,
    BlackBoxLabel: Eq + Clone + Send + Sync,
{
    /*
    the most obvious implementation of Frobenius is FrobeniusMorphism itself
    */
    fn interpret_unit(z: Lambda) -> Self {
        FrobeniusOperation::Unit(z).into()
    }
    fn interpret_counit(z: Lambda) -> Self {
        FrobeniusOperation::Counit(z).into()
    }
    fn interpret_multiplication(z: Lambda) -> Self {
        FrobeniusOperation::Multiplication(z).into()
    }
    fn interpret_comultiplication(z: Lambda) -> Self {
        FrobeniusOperation::Comultiplication(z).into()
    }

    fn basic_interpret<F>(
        single_step: &FrobeniusOperation<Lambda, BlackBoxLabel>,
        _black_box_interpreter: &F,
    ) -> Result<Self, CatgraphError>
    where
        F: Fn(&BlackBoxLabel, &[Lambda], &[Lambda]) -> Result<Self, CatgraphError>,
    {
        /*
        ignores black_box_interpreter as if it was just the simple
        |label,src,tgt| Ok(FrobeniusOperation::UnSpecifiedBox(label, src, tgt))
        */
        Ok(single_step.clone().into())
    }

    fn interpret_frob<F>(
        morphism: &FrobeniusMorphism<Lambda, BlackBoxLabel>,
        _black_box_interpreter: &F,
    ) -> Result<Self, CatgraphError>
    where
        F: Fn(&BlackBoxLabel, &[Lambda], &[Lambda]) -> Result<Self, CatgraphError>,
    {
        /*
        ignores black_box_interpreter as if it was just the simple
        |label,src,tgt| Ok(FrobeniusOperation::UnSpecifiedBox(label, src, tgt))
        */
        Ok(morphism.clone())
    }
}

impl<Lambda, BlackBoxLabel, T>
    InterpretableMorphism<FrobeniusMorphism<Lambda, BlackBoxLabel>, Lambda, BlackBoxLabel> for T
where
    Lambda: Eq + Copy + Debug + Send + Sync,
    BlackBoxLabel: Eq + Clone + Send + Sync,
    T: Frobenius<Lambda, BlackBoxLabel>,
{
    fn interpret<F>(
        gens: &FrobeniusMorphism<Lambda, BlackBoxLabel>,
        black_box_interpreter: F,
    ) -> Result<Self, CatgraphError>
    where
        F: Fn(&BlackBoxLabel, &[Lambda], &[Lambda]) -> Result<Self, CatgraphError>,
    {
        Self::interpret_frob(gens, &black_box_interpreter)
    }
}
