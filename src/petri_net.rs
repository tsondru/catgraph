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

    /// Returns indices of transitions enabled under the given marking.
    pub fn enabled(&self, marking: &Marking) -> Vec<usize> {
        self.transitions
            .iter()
            .enumerate()
            .filter(|(_, t)| t.pre.iter().all(|(p, w)| marking.get(*p) >= *w))
            .map(|(i, _)| i)
            .collect()
    }

    /// Fire a transition, returning the new marking. Pure.
    /// Fails if out of bounds or not enabled.
    pub fn fire(
        &self,
        transition: usize,
        marking: &Marking,
    ) -> Result<Marking, CatgraphError> {
        if transition >= self.transitions.len() {
            return Err(CatgraphError::PetriNet {
                message: format!(
                    "transition {} out of bounds (net has {} transitions)",
                    transition,
                    self.transitions.len()
                ),
            });
        }
        let t = &self.transitions[transition];
        for (p, w) in &t.pre {
            if marking.get(*p) < *w {
                return Err(CatgraphError::PetriNet {
                    message: format!(
                        "transition {} not enabled under current marking",
                        transition
                    ),
                });
            }
        }
        let mut result = marking.clone();
        for (p, w) in &t.pre {
            let c = result.get(*p) - w;
            result.set(*p, c);
        }
        for (p, w) in &t.post {
            let c = result.get(*p) + w;
            result.set(*p, c);
        }
        Ok(result)
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

    // Helper: 2H2 + O2 -> 2H2O
    fn combustion_net() -> PetriNet<char> {
        PetriNet::new(
            vec!['H', 'O', 'W'],
            vec![Transition::new(vec![(0, 2), (1, 1)], vec![(2, 2)])],
        )
    }

    #[test]
    fn enabled_sufficient_tokens() {
        let net = combustion_net();
        let m = Marking::from_vec(vec![(0, 4), (1, 2)]);
        assert_eq!(net.enabled(&m), vec![0]);
    }

    #[test]
    fn enabled_insufficient_tokens() {
        let net = combustion_net();
        let m = Marking::from_vec(vec![(0, 1), (1, 2)]);
        assert!(net.enabled(&m).is_empty());
    }

    #[test]
    fn fire_success() {
        let net = combustion_net();
        let m = Marking::from_vec(vec![(0, 2), (1, 1)]);
        let result = net.fire(0, &m).unwrap();
        assert_eq!(result.get(0), 0);
        assert_eq!(result.get(1), 0);
        assert_eq!(result.get(2), 2);
    }

    #[test]
    fn fire_not_enabled() {
        let net = combustion_net();
        let m = Marking::from_vec(vec![(0, 1)]);
        assert!(matches!(
            net.fire(0, &m).unwrap_err(),
            CatgraphError::PetriNet { .. }
        ));
    }

    #[test]
    fn fire_out_of_bounds() {
        let net = combustion_net();
        let m = Marking::new();
        assert!(matches!(
            net.fire(5, &m).unwrap_err(),
            CatgraphError::PetriNet { .. }
        ));
    }

    #[test]
    fn fire_preserves_other_places() {
        let net = combustion_net();
        let m = Marking::from_vec(vec![(0, 4), (1, 2), (2, 3)]);
        let result = net.fire(0, &m).unwrap();
        assert_eq!(result.get(0), 2);
        assert_eq!(result.get(1), 1);
        assert_eq!(result.get(2), 5);
    }
}
