mod morphism_system;
mod operations;
mod trait_impl;

pub use morphism_system::{Contains, InterpretableMorphism, MorphismSystem};
pub use operations::{FrobeniusMorphism, FrobeniusOperation, from_decomposition, special_frobenius_morphism};
pub use trait_impl::Frobenius;
