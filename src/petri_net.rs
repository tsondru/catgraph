//! Place/transition Petri nets with Lambda-typed places.
//!
//! Firing is pure (returns new marking). Composition via cospan bridge
//! connects to catgraph's pushout and monoidal infrastructure.

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
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

impl Hash for Marking {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut entries: Vec<_> = self.tokens.iter().collect();
        entries.sort_by_key(|(k, _)| *k);
        for (k, v) in entries {
            k.hash(state);
            v.hash(state);
        }
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

    /// Pre-arc weight for a (place, transition) pair. Zero if no arc.
    pub fn arc_weight_pre(&self, place: usize, transition: usize) -> u64 {
        self.transitions
            .get(transition)
            .map(|t| {
                t.pre
                    .iter()
                    .filter(|(p, _)| *p == place)
                    .map(|(_, w)| w)
                    .sum()
            })
            .unwrap_or(0)
    }

    /// Post-arc weight for a (place, transition) pair. Zero if no arc.
    pub fn arc_weight_post(&self, place: usize, transition: usize) -> u64 {
        self.transitions
            .get(transition)
            .map(|t| {
                t.post
                    .iter()
                    .filter(|(p, _)| *p == place)
                    .map(|(_, w)| w)
                    .sum()
            })
            .unwrap_or(0)
    }

    /// Places with no post-arcs from any transition (no transition produces tokens here).
    pub fn source_places(&self) -> Vec<usize> {
        (0..self.places.len())
            .filter(|p| {
                !self
                    .transitions
                    .iter()
                    .any(|t| t.post.iter().any(|(tp, _)| tp == p))
            })
            .collect()
    }

    /// Places with no pre-arcs to any transition (no transition consumes tokens from here).
    pub fn sink_places(&self) -> Vec<usize> {
        (0..self.places.len())
            .filter(|p| {
                !self
                    .transitions
                    .iter()
                    .any(|t| t.pre.iter().any(|(tp, _)| tp == p))
            })
            .collect()
    }

    /// All markings reachable from the initial marking within max_depth firing steps (BFS).
    pub fn reachable(&self, initial: &Marking, max_depth: usize) -> Vec<Marking> {
        let mut visited: HashSet<Marking> = HashSet::new();
        let mut queue: VecDeque<(Marking, usize)> = VecDeque::new();
        visited.insert(initial.clone());
        queue.push_back((initial.clone(), 0));
        while let Some((marking, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            for t in self.enabled(&marking) {
                if let Ok(next) = self.fire(t, &marking) {
                    if visited.insert(next.clone()) {
                        queue.push_back((next, depth + 1));
                    }
                }
            }
        }
        visited.into_iter().collect()
    }

    /// True if the target marking is reachable from initial within max_depth steps.
    pub fn can_reach(&self, initial: &Marking, target: &Marking, max_depth: usize) -> bool {
        if initial == target {
            return true;
        }
        let mut visited: HashSet<Marking> = HashSet::new();
        let mut queue: VecDeque<(Marking, usize)> = VecDeque::new();
        visited.insert(initial.clone());
        queue.push_back((initial.clone(), 0));
        while let Some((marking, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            for t in self.enabled(&marking) {
                if let Ok(next) = self.fire(t, &marking) {
                    if &next == target {
                        return true;
                    }
                    if visited.insert(next.clone()) {
                        queue.push_back((next, depth + 1));
                    }
                }
            }
        }
        false
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

    #[test]
    fn arc_weight_pre_existing() {
        let net = combustion_net();
        assert_eq!(net.arc_weight_pre(0, 0), 2);
        assert_eq!(net.arc_weight_pre(1, 0), 1);
    }

    #[test]
    fn arc_weight_pre_missing() {
        let net = combustion_net();
        assert_eq!(net.arc_weight_pre(2, 0), 0);
    }

    #[test]
    fn arc_weight_post_existing() {
        let net = combustion_net();
        assert_eq!(net.arc_weight_post(2, 0), 2);
    }

    #[test]
    fn arc_weight_post_missing() {
        let net = combustion_net();
        assert_eq!(net.arc_weight_post(0, 0), 0);
    }

    #[test]
    fn source_places_combustion() {
        let net = combustion_net();
        let sources = net.source_places();
        assert!(sources.contains(&0));
        assert!(sources.contains(&1));
        assert!(!sources.contains(&2));
    }

    #[test]
    fn sink_places_combustion() {
        let net = combustion_net();
        let sinks = net.sink_places();
        assert!(sinks.contains(&2));
        assert!(!sinks.contains(&0));
    }

    #[test]
    fn reachable_single_step() {
        let net = combustion_net();
        let m0 = Marking::from_vec(vec![(0, 2), (1, 1)]);
        let reachable = net.reachable(&m0, 1);
        assert_eq!(reachable.len(), 2);
        assert!(reachable.contains(&m0));
        assert!(reachable.contains(&Marking::from_vec(vec![(2, 2)])));
    }

    #[test]
    fn reachable_no_enabled() {
        let net = combustion_net();
        let m0 = Marking::new();
        let reachable = net.reachable(&m0, 10);
        assert_eq!(reachable.len(), 1);
    }

    #[test]
    fn can_reach_true() {
        let net = combustion_net();
        let m0 = Marking::from_vec(vec![(0, 2), (1, 1)]);
        let target = Marking::from_vec(vec![(2, 2)]);
        assert!(net.can_reach(&m0, &target, 5));
    }

    #[test]
    fn can_reach_false() {
        let net = combustion_net();
        let m0 = Marking::from_vec(vec![(0, 2), (1, 1)]);
        let target = Marking::from_vec(vec![(2, 99)]);
        assert!(!net.can_reach(&m0, &target, 10));
    }

    #[test]
    fn reachable_multi_step() {
        let net = combustion_net();
        let m0 = Marking::from_vec(vec![(0, 4), (1, 2)]);
        let reachable = net.reachable(&m0, 3);
        assert_eq!(reachable.len(), 3);
        assert!(reachable.contains(&Marking::from_vec(vec![(2, 4)])));
    }
}
