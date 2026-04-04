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

    /// Construct a single-transition Petri net from a cospan.
    /// Left leg multiplicities become pre-arc weights, right leg become post-arc weights.
    pub fn from_cospan(cospan: &Cospan<Lambda>) -> Self {
        let places = cospan.middle().to_vec();
        let mut pre_counts: HashMap<usize, u64> = HashMap::new();
        for &idx in cospan.left_to_middle() {
            *pre_counts.entry(idx).or_insert(0) += 1;
        }
        let mut post_counts: HashMap<usize, u64> = HashMap::new();
        for &idx in cospan.right_to_middle() {
            *post_counts.entry(idx).or_insert(0) += 1;
        }
        let pre: Vec<(usize, u64)> = pre_counts.into_iter().collect();
        let post: Vec<(usize, u64)> = post_counts.into_iter().collect();
        Self::new(places, vec![Transition::new(pre, post)])
    }

    /// Convert a single transition to its cospan representation.
    /// Pre-arc weights become left leg multiplicities, post-arc weights become right leg multiplicities.
    pub fn transition_as_cospan(&self, transition: usize) -> Cospan<Lambda> {
        let t = &self.transitions[transition];
        let mut left = Vec::new();
        for (p, w) in &t.pre {
            for _ in 0..*w {
                left.push(*p);
            }
        }
        let mut right = Vec::new();
        for (p, w) in &t.post {
            for _ in 0..*w {
                right.push(*p);
            }
        }
        Cospan::new(left, right, self.places.clone())
    }

    /// Parallel composition: disjoint union of places and transitions.
    pub fn parallel(&self, other: &Self) -> Self {
        let offset = self.places.len();
        let mut places = self.places.clone();
        places.extend_from_slice(&other.places);
        let mut transitions = self.transitions.clone();
        for t in &other.transitions {
            let pre: Vec<(usize, u64)> = t.pre.iter().map(|(p, w)| (p + offset, *w)).collect();
            let post: Vec<(usize, u64)> = t.post.iter().map(|(p, w)| (p + offset, *w)).collect();
            transitions.push(Transition::new(pre, post));
        }
        Self::new(places, transitions)
    }

    /// Sequential composition: merge sink places of self with source places of other by Lambda match.
    pub fn sequential(&self, other: &Self) -> Result<Self, CatgraphError> {
        let self_sinks = self.sink_places();
        let other_sources = other.source_places();
        let mut merge_map: HashMap<usize, usize> = HashMap::new();
        let mut used_sinks: HashSet<usize> = HashSet::new();
        for &os in &other_sources {
            for &ss in &self_sinks {
                if !used_sinks.contains(&ss) && self.places[ss] == other.places[os] {
                    merge_map.insert(os, ss);
                    used_sinks.insert(ss);
                    break;
                }
            }
        }
        let mut places = self.places.clone();
        let mut other_index_map: Vec<usize> = Vec::with_capacity(other.places.len());
        for (i, &lambda) in other.places.iter().enumerate() {
            if let Some(&merged_idx) = merge_map.get(&i) {
                other_index_map.push(merged_idx);
            } else {
                other_index_map.push(places.len());
                places.push(lambda);
            }
        }
        let mut transitions = self.transitions.clone();
        for t in &other.transitions {
            let pre: Vec<(usize, u64)> =
                t.pre.iter().map(|(p, w)| (other_index_map[*p], *w)).collect();
            let post: Vec<(usize, u64)> =
                t.post.iter().map(|(p, w)| (other_index_map[*p], *w)).collect();
            transitions.push(Transition::new(pre, post));
        }
        Ok(Self::new(places, transitions))
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

    #[test]
    fn from_cospan_single_transition() {
        let cospan: Cospan<char> = Cospan::new(vec![0, 1, 1, 1], vec![2, 2], vec!['N', 'H', 'A']);
        let net = PetriNet::from_cospan(&cospan);
        assert_eq!(net.place_count(), 3);
        assert_eq!(net.transition_count(), 1);
        assert_eq!(net.arc_weight_pre(0, 0), 1);
        assert_eq!(net.arc_weight_pre(1, 0), 3);
        assert_eq!(net.arc_weight_post(2, 0), 2);
    }

    #[test]
    fn transition_as_cospan_roundtrip() {
        let net = combustion_net();
        let cospan = net.transition_as_cospan(0);
        let roundtrip = PetriNet::from_cospan(&cospan);
        assert_eq!(roundtrip.place_count(), net.place_count());
        assert_eq!(roundtrip.arc_weight_pre(0, 0), net.arc_weight_pre(0, 0));
        assert_eq!(roundtrip.arc_weight_pre(1, 0), net.arc_weight_pre(1, 0));
        assert_eq!(roundtrip.arc_weight_post(2, 0), net.arc_weight_post(2, 0));
    }

    #[test]
    fn parallel_composition() {
        let a: PetriNet<char> = PetriNet::new(
            vec!['a', 'b'],
            vec![Transition::new(vec![(0, 1)], vec![(1, 1)])],
        );
        let b: PetriNet<char> = PetriNet::new(
            vec!['c', 'd'],
            vec![Transition::new(vec![(0, 1)], vec![(1, 1)])],
        );
        let combined = a.parallel(&b);
        assert_eq!(combined.place_count(), 4);
        assert_eq!(combined.transition_count(), 2);
        assert_eq!(combined.arc_weight_pre(2, 1), 1);
        assert_eq!(combined.arc_weight_post(3, 1), 1);
    }

    #[test]
    fn sequential_composition() {
        let a: PetriNet<char> = PetriNet::new(
            vec!['a', 'b'],
            vec![Transition::new(vec![(0, 1)], vec![(1, 1)])],
        );
        let b: PetriNet<char> = PetriNet::new(
            vec!['b', 'c'],
            vec![Transition::new(vec![(0, 1)], vec![(1, 1)])],
        );
        let composed = a.sequential(&b).unwrap();
        assert_eq!(composed.place_count(), 3);
        assert_eq!(composed.transition_count(), 2);
    }

    #[test]
    fn sequential_no_matching_boundary() {
        let a: PetriNet<char> = PetriNet::new(
            vec!['a', 'b'],
            vec![Transition::new(vec![(0, 1)], vec![(1, 1)])],
        );
        let b: PetriNet<char> = PetriNet::new(
            vec!['x', 'y'],
            vec![Transition::new(vec![(0, 1)], vec![(1, 1)])],
        );
        let composed = a.sequential(&b).unwrap();
        assert_eq!(composed.place_count(), 4);
    }
}
