//! Place/transition Petri nets with Lambda-typed places.
//!
//! Firing is pure (returns new marking). Composition via cospan bridge
//! connects to catgraph's pushout and monoidal infrastructure.

use std::collections::HashMap;
use std::fmt::Debug;

use crate::cospan::Cospan;
use crate::errors::CatgraphError;

/// A single transition: pre-set and post-set as weighted arcs over place indices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transition {
    pre: Vec<(usize, u64)>,
    post: Vec<(usize, u64)>,
}

impl Transition {
    pub fn new(pre: Vec<(usize, u64)>, post: Vec<(usize, u64)>) -> Self {
        Self { pre, post }
    }
    pub fn pre(&self) -> &[(usize, u64)] {
        &self.pre
    }
    pub fn post(&self) -> &[(usize, u64)] {
        &self.post
    }
}

/// Token assignment: place index to count. Sparse (only nonzero entries stored).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Marking {
    tokens: HashMap<usize, u64>,
}

impl Marking {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
        }
    }
    pub fn from_vec(pairs: Vec<(usize, u64)>) -> Self {
        let tokens: HashMap<usize, u64> = pairs.into_iter().filter(|(_, c)| *c > 0).collect();
        Self { tokens }
    }
    pub fn set(&mut self, place: usize, count: u64) {
        if count == 0 {
            self.tokens.remove(&place);
        } else {
            self.tokens.insert(place, count);
        }
    }
    pub fn get(&self, place: usize) -> u64 {
        self.tokens.get(&place).copied().unwrap_or(0)
    }
    pub fn tokens(&self) -> &HashMap<usize, u64> {
        &self.tokens
    }
}

impl Default for Marking {
    fn default() -> Self {
        Self::new()
    }
}

/// A place/transition Petri net with Lambda-typed places.
///
/// Future: colored tokens (typed multisets) and weighted tokens (semiring-valued
/// markings for stochastic/continuous/timed Petri nets connecting to magnitude enrichment).
#[derive(Clone, Debug)]
pub struct PetriNet<Lambda: Sized + Eq + Copy + Debug> {
    places: Vec<Lambda>,
    transitions: Vec<Transition>,
}

impl<Lambda> PetriNet<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
{
    pub fn new(places: Vec<Lambda>, transitions: Vec<Transition>) -> Self {
        Self {
            places,
            transitions,
        }
    }
    pub fn places(&self) -> &[Lambda] {
        &self.places
    }
    pub fn transitions(&self) -> &[Transition] {
        &self.transitions
    }
    pub fn place_count(&self) -> usize {
        self.places.len()
    }
    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn marking_new_is_empty() {
        let m = Marking::new();
        assert_eq!(m.get(0), 0);
        assert!(m.tokens().is_empty());
    }

    #[test]
    fn marking_from_vec_filters_zeros() {
        let m = Marking::from_vec(vec![(0, 3), (1, 0), (2, 1)]);
        assert_eq!(m.get(0), 3);
        assert_eq!(m.get(1), 0);
        assert_eq!(m.get(2), 1);
        assert_eq!(m.tokens().len(), 2);
    }

    #[test]
    fn marking_set_and_get() {
        let mut m = Marking::new();
        m.set(0, 5);
        assert_eq!(m.get(0), 5);
        m.set(0, 0);
        assert_eq!(m.get(0), 0);
        assert!(m.tokens().is_empty());
    }

    #[test]
    fn petri_net_construction() {
        let net: PetriNet<char> = PetriNet::new(
            vec!['H', 'O', 'W'],
            vec![Transition::new(vec![(0, 2), (1, 1)], vec![(2, 2)])],
        );
        assert_eq!(net.place_count(), 3);
        assert_eq!(net.transition_count(), 1);
    }

    #[test]
    fn transition_accessors() {
        let t = Transition::new(vec![(0, 1), (1, 2)], vec![(2, 3)]);
        assert_eq!(t.pre(), &[(0, 1), (1, 2)]);
        assert_eq!(t.post(), &[(2, 3)]);
    }
}
