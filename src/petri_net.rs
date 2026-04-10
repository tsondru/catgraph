//! Place/transition Petri nets with Lambda-typed places.
//!
//! A Petri net is a bipartite directed graph of *places* (holding tokens) and
//! *transitions* (consuming/producing tokens). Each transition has weighted
//! pre-arcs (inputs) and post-arcs (outputs); firing a transition is an atomic
//! consume-then-produce step. Markings are sparse token assignments.
//!
//! Firing is pure: [`PetriNet::fire`] returns a new [`Marking`] without mutating
//! the original. Reachability analysis ([`PetriNet::reachable`], [`PetriNet::can_reach`])
//! uses bounded BFS over the firing graph.
//!
//! ## Cospan bridge
//!
//! [`PetriNet::from_cospan`] and [`PetriNet::transition_as_cospan`] connect Petri
//! nets to catgraph's pushout and monoidal infrastructure: left-leg multiplicities
//! become pre-arc weights and right-leg multiplicities become post-arc weights.
//! This gives Petri nets source/target (cospan) semantics where places are the
//! middle set and transitions are the morphisms.
//!
//! ## Composition
//!
//! [`PetriNet::parallel`] takes the disjoint union of places and transitions
//! (monoidal product). [`PetriNet::sequential`] merges sink places of one net
//! with Lambda-matching source places of another.
//!
//! ## MorphismSystem integration
//!
//! [`PetriNetDescription`] provides a compositional description language for
//! building Petri nets via [`MorphismSystem`](crate::frobenius::MorphismSystem).
//! Register primitive nets, then compose them by name using `Parallel` and
//! `Sequential` variants. Black boxes are resolved via topological DAG sort.
//!
//! See also `examples/petri_net.rs` for chemical reaction modelling.

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::fmt::Debug;

use num::ToPrimitive;
use rust_decimal::Decimal;

use crate::cospan::Cospan;
use crate::errors::CatgraphError;
use crate::frobenius::{Contains, InterpretableMorphism};

/// A single transition in a Petri net.
///
/// A transition fires by consuming tokens from its pre-set and producing tokens
/// into its post-set. Each arc carries a weight indicating how many tokens are
/// consumed or produced at the connected place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transition {
    /// Pre-arcs: `(place_index, weight)` pairs specifying how many tokens
    /// must be present (and are consumed) at each input place.
    pre: Vec<(usize, Decimal)>,
    /// Post-arcs: `(place_index, weight)` pairs specifying how many tokens
    /// are produced at each output place.
    post: Vec<(usize, Decimal)>,
}

impl Transition {
    /// Construct a transition from its pre-arc and post-arc weight lists.
    #[must_use] 
    pub fn new(pre: Vec<(usize, Decimal)>, post: Vec<(usize, Decimal)>) -> Self {
        Self { pre, post }
    }

    /// The pre-arcs (input places and their weights).
    #[must_use] 
    pub fn pre(&self) -> &[(usize, Decimal)] {
        &self.pre
    }

    /// The post-arcs (output places and their weights).
    #[must_use] 
    pub fn post(&self) -> &[(usize, Decimal)] {
        &self.post
    }
}

/// A token distribution across places in a Petri net.
///
/// Stored as a sparse map from place index to token count; places with zero
/// tokens are not stored. Two markings are equal iff they assign the same
/// token count to every place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Marking {
    /// Sparse map: place index → nonzero token count.
    tokens: HashMap<usize, Decimal>,
}

impl Marking {
    /// Create an empty marking (no tokens anywhere).
    #[must_use] 
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
        }
    }

    /// Create a marking from `(place_index, count)` pairs.
    /// Pairs with count zero are silently dropped.
    #[must_use] 
    pub fn from_vec(pairs: Vec<(usize, Decimal)>) -> Self {
        let tokens: HashMap<usize, Decimal> = pairs.into_iter().filter(|(_, c)| !c.is_zero()).collect();
        Self { tokens }
    }

    /// Set the token count at a place. Setting to zero removes the entry.
    pub fn set(&mut self, place: usize, count: Decimal) {
        if count.is_zero() {
            self.tokens.remove(&place);
        } else {
            self.tokens.insert(place, count);
        }
    }

    /// Get the token count at a place (zero if absent).
    #[must_use] 
    pub fn get(&self, place: usize) -> Decimal {
        self.tokens.get(&place).copied().unwrap_or(Decimal::ZERO)
    }

    /// The underlying sparse token map.
    #[must_use] 
    pub fn tokens(&self) -> &HashMap<usize, Decimal> {
        &self.tokens
    }
}

impl Default for Marking {
    fn default() -> Self {
        Self::new()
    }
}

/// Hash by sorting entries by place index so that equal markings always
/// produce the same hash regardless of `HashMap` iteration order.
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
/// The type parameter `Lambda` labels each place (e.g. `char` for chemical
/// species like `'H'`, `'O'`, `'W'`). The net stores a flat vector of places
/// and a vector of [`Transition`]s whose arc indices refer into the places vector.
///
/// All firing operations are pure: they return new [`Marking`]s without mutating
/// the net or the input marking.
///
/// Future: colored tokens (typed multisets) and weighted tokens (semiring-valued
/// markings for stochastic/continuous/timed Petri nets connecting to magnitude enrichment).
#[derive(Clone, Debug)]
pub struct PetriNet<Lambda: Sized + Eq + Copy + Debug> {
    /// Lambda-typed places, indexed by position.
    places: Vec<Lambda>,
    /// Transitions with weighted arcs referencing place indices.
    transitions: Vec<Transition>,
}

impl<Lambda> PetriNet<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
{
    /// Construct a Petri net from its places and transitions.
    #[must_use] 
    pub fn new(places: Vec<Lambda>, transitions: Vec<Transition>) -> Self {
        Self {
            places,
            transitions,
        }
    }

    /// The Lambda-typed places.
    #[must_use] 
    pub fn places(&self) -> &[Lambda] {
        &self.places
    }

    /// The transitions in this net.
    #[must_use] 
    pub fn transitions(&self) -> &[Transition] {
        &self.transitions
    }

    /// Number of places.
    #[must_use] 
    pub fn place_count(&self) -> usize {
        self.places.len()
    }

    /// Number of transitions.
    #[must_use] 
    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }

    /// Returns indices of transitions enabled under the given marking.
    ///
    /// A transition is enabled when every pre-arc place holds at least as many
    /// tokens as the arc weight requires.
    #[must_use] 
    pub fn enabled(&self, marking: &Marking) -> Vec<usize> {
        self.transitions
            .iter()
            .enumerate()
            .filter(|(_, t)| t.pre.iter().all(|(p, w)| marking.get(*p) >= *w))
            .map(|(i, _)| i)
            .collect()
    }

    /// Fire a transition, returning the new marking.
    ///
    /// Subtracts pre-arc weights and adds post-arc weights. The operation is
    /// pure: the input marking is not modified.
    ///
    /// # Errors
    /// Returns [`CatgraphError::PetriNet`] if the transition index is out of
    /// bounds or the transition is not enabled under the given marking.
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
                        "transition {transition} not enabled under current marking"
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
    #[must_use] 
    pub fn arc_weight_pre(&self, place: usize, transition: usize) -> Decimal {
        self.transitions
            .get(transition)
            .map_or(Decimal::ZERO, |t| {
                t.pre
                    .iter()
                    .filter(|(p, _)| *p == place)
                    .map(|(_, w)| w)
                    .sum()
            })
    }

    /// Post-arc weight for a (place, transition) pair. Zero if no arc.
    #[must_use] 
    pub fn arc_weight_post(&self, place: usize, transition: usize) -> Decimal {
        self.transitions
            .get(transition)
            .map_or(Decimal::ZERO, |t| {
                t.post
                    .iter()
                    .filter(|(p, _)| *p == place)
                    .map(|(_, w)| w)
                    .sum()
            })
    }

    /// Places with no post-arcs from any transition (no transition produces tokens here).
    #[must_use] 
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
    #[must_use] 
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

    /// All markings reachable from the initial marking within `max_depth` firing steps.
    ///
    /// Performs a breadth-first search over the firing graph. The returned set
    /// always includes the initial marking itself.
    #[must_use] 
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
                if let Ok(next) = self.fire(t, &marking)
                    && visited.insert(next.clone())
                {
                    queue.push_back((next, depth + 1));
                }
            }
        }
        visited.into_iter().collect()
    }

    /// True if the target marking is reachable from `initial` within `max_depth` steps.
    ///
    /// Short-circuits as soon as the target is found. Returns `true` immediately
    /// if `initial == target`.
    #[must_use] 
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
    ///
    /// The cospan's middle set becomes the places. Left-leg multiplicities
    /// (how many domain nodes map to each middle node) become pre-arc weights;
    /// right-leg multiplicities become post-arc weights. This establishes the
    /// cospan bridge between Petri net firing semantics and categorical composition.
    #[must_use] 
    pub fn from_cospan(cospan: &Cospan<Lambda>) -> Self {
        let places = cospan.middle().to_vec();
        let mut pre_counts: HashMap<usize, Decimal> = HashMap::new();
        for &idx in cospan.left_to_middle() {
            *pre_counts.entry(idx).or_insert(Decimal::ZERO) += Decimal::ONE;
        }
        let mut post_counts: HashMap<usize, Decimal> = HashMap::new();
        for &idx in cospan.right_to_middle() {
            *post_counts.entry(idx).or_insert(Decimal::ZERO) += Decimal::ONE;
        }
        let pre: Vec<(usize, Decimal)> = pre_counts.into_iter().collect();
        let post: Vec<(usize, Decimal)> = post_counts.into_iter().collect();
        Self::new(places, vec![Transition::new(pre, post)])
    }

    /// Convert a single transition to its cospan representation.
    ///
    /// Each pre-arc weight becomes a multiplicity in the left (domain) leg,
    /// and each post-arc weight becomes a multiplicity in the right (codomain)
    /// leg. Inverse of [`PetriNet::from_cospan`] for single-transition nets.
    ///
    /// # Panics
    ///
    /// Panics if any arc weight is not representable as `u64`.
    #[must_use]
    pub fn transition_as_cospan(&self, transition: usize) -> Cospan<Lambda> {
        let t = &self.transitions[transition];
        let mut left = Vec::new();
        for (p, w) in &t.pre {
            let count = w.to_u64().expect("integer weight for cospan expansion");
            for _ in 0..count {
                left.push(*p);
            }
        }
        let mut right = Vec::new();
        for (p, w) in &t.post {
            let count = w.to_u64().expect("integer weight for cospan expansion");
            for _ in 0..count {
                right.push(*p);
            }
        }
        Cospan::new(left, right, self.places.clone())
    }

    /// Parallel composition (monoidal product): disjoint union of places and transitions.
    ///
    /// Place indices in `other` are shifted by `self.place_count()`. Neither net
    /// is modified; a new combined net is returned.
    #[must_use] 
    pub fn parallel(&self, other: &Self) -> Self {
        let offset = self.places.len();
        let mut places = self.places.clone();
        places.extend_from_slice(&other.places);
        let mut transitions = self.transitions.clone();
        for t in &other.transitions {
            let pre: Vec<(usize, Decimal)> = t.pre.iter().map(|(p, w)| (p + offset, *w)).collect();
            let post: Vec<(usize, Decimal)> = t.post.iter().map(|(p, w)| (p + offset, *w)).collect();
            transitions.push(Transition::new(pre, post));
        }
        Self::new(places, transitions)
    }

    /// Sequential composition: merge sink places of `self` with source places of `other`.
    ///
    /// Matching is by Lambda equality: each unmatched source place in `other` is
    /// paired with an unused sink place in `self` that carries the same Lambda type.
    /// Unmatched places from `other` are appended as new places.
    ///
    /// # Errors
    ///
    /// Returns [`CatgraphError`] if boundary place types don't match between nets.
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
            let pre: Vec<(usize, Decimal)> =
                t.pre.iter().map(|(p, w)| (other_index_map[*p], *w)).collect();
            let post: Vec<(usize, Decimal)> =
                t.post.iter().map(|(p, w)| (other_index_map[*p], *w)).collect();
            transitions.push(Transition::new(pre, post));
        }
        Ok(Self::new(places, transitions))
    }
}

/// A compositional description of a Petri net for use with
/// [`MorphismSystem`](crate::frobenius::MorphismSystem).
///
/// Each variant describes how to build a `PetriNet<Lambda>`:
/// - `Primitive` wraps a concrete net (leaf in the DAG).
/// - `Parallel` takes the monoidal product (disjoint union) of two named nets.
/// - `Sequential` composes two named nets by merging matching boundary places.
/// - `BlackBox` is an opaque sub-net resolved by the `MorphismSystem` interpreter.
#[derive(Clone, Debug)]
pub enum PetriNetDescription<Lambda: Sized + Eq + Copy + Debug, BlackBoxLabel: Eq + Clone> {
    /// A concrete Petri net (leaf node in the composition DAG).
    Primitive(PetriNet<Lambda>),
    /// Parallel composition (monoidal product) of two named sub-nets.
    Parallel(BlackBoxLabel, BlackBoxLabel),
    /// Sequential composition of two named sub-nets (sink/source merging).
    Sequential(BlackBoxLabel, BlackBoxLabel),
    /// An opaque named sub-net with declared source and sink place types.
    BlackBox(BlackBoxLabel, Vec<Lambda>, Vec<Lambda>),
}

impl<Lambda, BlackBoxLabel> Contains<BlackBoxLabel>
    for PetriNetDescription<Lambda, BlackBoxLabel>
where
    Lambda: Sized + Eq + Copy + Debug,
    BlackBoxLabel: Eq + Clone,
{
    fn contained_labels(&self) -> Vec<BlackBoxLabel> {
        match self {
            Self::Primitive(_) => vec![],
            Self::Parallel(a, b) | Self::Sequential(a, b) => vec![a.clone(), b.clone()],
            Self::BlackBox(label, _, _) => vec![label.clone()],
        }
    }
}

impl<Lambda, BlackBoxLabel> InterpretableMorphism<PetriNetDescription<Lambda, BlackBoxLabel>, Lambda, BlackBoxLabel>
    for PetriNet<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
    BlackBoxLabel: Eq + Clone + Debug,
{
    fn interpret<F>(
        desc: &PetriNetDescription<Lambda, BlackBoxLabel>,
        black_box_interpreter: F,
    ) -> Result<Self, CatgraphError>
    where
        F: Fn(&BlackBoxLabel, &[Lambda], &[Lambda]) -> Result<Self, CatgraphError>,
    {
        match desc {
            PetriNetDescription::Primitive(net) => Ok(net.clone()),
            PetriNetDescription::Parallel(a, b) => {
                let net_a = black_box_interpreter(a, &[], &[])?;
                let net_b = black_box_interpreter(b, &[], &[])?;
                Ok(net_a.parallel(&net_b))
            }
            PetriNetDescription::Sequential(a, b) => {
                let net_a = black_box_interpreter(a, &[], &[])?;
                let net_b = black_box_interpreter(b, &[], &[])?;
                net_a.sequential(&net_b)
            }
            PetriNetDescription::BlackBox(label, src, tgt) => {
                black_box_interpreter(label, src, tgt)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rust_decimal::Decimal;

    /// Shorthand for `Decimal::from(n)`.
    fn d(n: i64) -> Decimal {
        Decimal::from(n)
    }

    #[test]
    fn marking_new_is_empty() {
        let m = Marking::new();
        assert_eq!(m.get(0), Decimal::ZERO);
        assert!(m.tokens().is_empty());
    }

    #[test]
    fn marking_from_vec_filters_zeros() {
        let m = Marking::from_vec(vec![(0, d(3)), (1, d(0)), (2, d(1))]);
        assert_eq!(m.get(0), d(3));
        assert_eq!(m.get(1), Decimal::ZERO);
        assert_eq!(m.get(2), d(1));
        assert_eq!(m.tokens().len(), 2);
    }

    #[test]
    fn marking_set_and_get() {
        let mut m = Marking::new();
        m.set(0, d(5));
        assert_eq!(m.get(0), d(5));
        m.set(0, Decimal::ZERO);
        assert_eq!(m.get(0), Decimal::ZERO);
        assert!(m.tokens().is_empty());
    }

    #[test]
    fn petri_net_construction() {
        let net: PetriNet<char> = PetriNet::new(
            vec!['H', 'O', 'W'],
            vec![Transition::new(vec![(0, d(2)), (1, d(1))], vec![(2, d(2))])],
        );
        assert_eq!(net.place_count(), 3);
        assert_eq!(net.transition_count(), 1);
    }

    #[test]
    fn transition_accessors() {
        let t = Transition::new(vec![(0, d(1)), (1, d(2))], vec![(2, d(3))]);
        assert_eq!(t.pre(), &[(0, d(1)), (1, d(2))]);
        assert_eq!(t.post(), &[(2, d(3))]);
    }

    // Helper: 2H2 + O2 -> 2H2O
    fn combustion_net() -> PetriNet<char> {
        PetriNet::new(
            vec!['H', 'O', 'W'],
            vec![Transition::new(vec![(0, d(2)), (1, d(1))], vec![(2, d(2))])],
        )
    }

    #[test]
    fn enabled_sufficient_tokens() {
        let net = combustion_net();
        let m = Marking::from_vec(vec![(0, d(4)), (1, d(2))]);
        assert_eq!(net.enabled(&m), vec![0]);
    }

    #[test]
    fn enabled_insufficient_tokens() {
        let net = combustion_net();
        let m = Marking::from_vec(vec![(0, d(1)), (1, d(2))]);
        assert!(net.enabled(&m).is_empty());
    }

    #[test]
    fn fire_success() {
        let net = combustion_net();
        let m = Marking::from_vec(vec![(0, d(2)), (1, d(1))]);
        let result = net.fire(0, &m).unwrap();
        assert_eq!(result.get(0), Decimal::ZERO);
        assert_eq!(result.get(1), Decimal::ZERO);
        assert_eq!(result.get(2), d(2));
    }

    #[test]
    fn fire_not_enabled() {
        let net = combustion_net();
        let m = Marking::from_vec(vec![(0, d(1))]);
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
        let m = Marking::from_vec(vec![(0, d(4)), (1, d(2)), (2, d(3))]);
        let result = net.fire(0, &m).unwrap();
        assert_eq!(result.get(0), d(2));
        assert_eq!(result.get(1), d(1));
        assert_eq!(result.get(2), d(5));
    }

    #[test]
    fn arc_weight_pre_existing() {
        let net = combustion_net();
        assert_eq!(net.arc_weight_pre(0, 0), d(2));
        assert_eq!(net.arc_weight_pre(1, 0), d(1));
    }

    #[test]
    fn arc_weight_pre_missing() {
        let net = combustion_net();
        assert_eq!(net.arc_weight_pre(2, 0), Decimal::ZERO);
    }

    #[test]
    fn arc_weight_post_existing() {
        let net = combustion_net();
        assert_eq!(net.arc_weight_post(2, 0), d(2));
    }

    #[test]
    fn arc_weight_post_missing() {
        let net = combustion_net();
        assert_eq!(net.arc_weight_post(0, 0), Decimal::ZERO);
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
        let m0 = Marking::from_vec(vec![(0, d(2)), (1, d(1))]);
        let reachable = net.reachable(&m0, 1);
        assert_eq!(reachable.len(), 2);
        assert!(reachable.contains(&m0));
        assert!(reachable.contains(&Marking::from_vec(vec![(2, d(2))])));
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
        let m0 = Marking::from_vec(vec![(0, d(2)), (1, d(1))]);
        let target = Marking::from_vec(vec![(2, d(2))]);
        assert!(net.can_reach(&m0, &target, 5));
    }

    #[test]
    fn can_reach_false() {
        let net = combustion_net();
        let m0 = Marking::from_vec(vec![(0, d(2)), (1, d(1))]);
        let target = Marking::from_vec(vec![(2, d(99))]);
        assert!(!net.can_reach(&m0, &target, 10));
    }

    #[test]
    fn reachable_multi_step() {
        let net = combustion_net();
        let m0 = Marking::from_vec(vec![(0, d(4)), (1, d(2))]);
        let reachable = net.reachable(&m0, 3);
        assert_eq!(reachable.len(), 3);
        assert!(reachable.contains(&Marking::from_vec(vec![(2, d(4))])));
    }

    #[test]
    fn from_cospan_single_transition() {
        let cospan: Cospan<char> = Cospan::new(vec![0, 1, 1, 1], vec![2, 2], vec!['N', 'H', 'A']);
        let net = PetriNet::from_cospan(&cospan);
        assert_eq!(net.place_count(), 3);
        assert_eq!(net.transition_count(), 1);
        assert_eq!(net.arc_weight_pre(0, 0), d(1));
        assert_eq!(net.arc_weight_pre(1, 0), d(3));
        assert_eq!(net.arc_weight_post(2, 0), d(2));
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
            vec![Transition::new(vec![(0, d(1))], vec![(1, d(1))])],
        );
        let b: PetriNet<char> = PetriNet::new(
            vec!['c', 'd'],
            vec![Transition::new(vec![(0, d(1))], vec![(1, d(1))])],
        );
        let combined = a.parallel(&b);
        assert_eq!(combined.place_count(), 4);
        assert_eq!(combined.transition_count(), 2);
        assert_eq!(combined.arc_weight_pre(2, 1), d(1));
        assert_eq!(combined.arc_weight_post(3, 1), d(1));
    }

    #[test]
    fn sequential_composition() {
        let a: PetriNet<char> = PetriNet::new(
            vec!['a', 'b'],
            vec![Transition::new(vec![(0, d(1))], vec![(1, d(1))])],
        );
        let b: PetriNet<char> = PetriNet::new(
            vec!['b', 'c'],
            vec![Transition::new(vec![(0, d(1))], vec![(1, d(1))])],
        );
        let composed = a.sequential(&b).unwrap();
        assert_eq!(composed.place_count(), 3);
        assert_eq!(composed.transition_count(), 2);
    }

    #[test]
    fn sequential_no_matching_boundary() {
        let a: PetriNet<char> = PetriNet::new(
            vec!['a', 'b'],
            vec![Transition::new(vec![(0, d(1))], vec![(1, d(1))])],
        );
        let b: PetriNet<char> = PetriNet::new(
            vec!['x', 'y'],
            vec![Transition::new(vec![(0, d(1))], vec![(1, d(1))])],
        );
        let composed = a.sequential(&b).unwrap();
        assert_eq!(composed.place_count(), 4);
    }

    // ── MorphismSystem integration tests ──

    use super::PetriNetDescription;
    use crate::frobenius::{Contains, MorphismSystem};

    type PetriNetSystem = MorphismSystem<String, char, PetriNetDescription<char, String>, PetriNet<char>>;

    #[test]
    fn petri_description_contains_labels() {
        let prim: PetriNetDescription<char, String> =
            PetriNetDescription::Primitive(combustion_net());
        assert!(prim.contained_labels().is_empty());

        let par: PetriNetDescription<char, String> =
            PetriNetDescription::Parallel("a".into(), "b".into());
        assert_eq!(par.contained_labels(), vec!["a", "b"]);

        let seq: PetriNetDescription<char, String> =
            PetriNetDescription::Sequential("x".into(), "y".into());
        assert_eq!(seq.contained_labels(), vec!["x", "y"]);

        let bb: PetriNetDescription<char, String> =
            PetriNetDescription::BlackBox("z".into(), vec!['H'], vec!['O']);
        assert_eq!(bb.contained_labels(), vec!["z"]);
    }

    #[test]
    fn morphism_system_primitive_resolves() {
        let mut sys: PetriNetSystem = MorphismSystem::new("comb".into());
        sys.add_definition_simple(
            "comb".into(),
            combustion_net(),
        ).unwrap();
        let resolved = sys.fill_black_boxes(None).unwrap();
        assert_eq!(resolved.place_count(), 3);
        assert_eq!(resolved.transition_count(), 1);
    }

    #[test]
    fn morphism_system_parallel_composition() {
        let mut sys: PetriNetSystem = MorphismSystem::new("combined".into());
        let net_a: PetriNet<char> = PetriNet::new(
            vec!['a', 'b'],
            vec![Transition::new(vec![(0, d(1))], vec![(1, d(1))])],
        );
        let net_b: PetriNet<char> = PetriNet::new(
            vec!['c', 'd'],
            vec![Transition::new(vec![(0, d(1))], vec![(1, d(1))])],
        );
        sys.add_definition_simple("left".into(), net_a).unwrap();
        sys.add_definition_simple("right".into(), net_b).unwrap();
        sys.add_definition_composite(
            "combined".into(),
            PetriNetDescription::Parallel("left".into(), "right".into()),
        ).unwrap();
        let resolved = sys.fill_black_boxes(None).unwrap();
        assert_eq!(resolved.place_count(), 4);
        assert_eq!(resolved.transition_count(), 2);
    }

    #[test]
    fn morphism_system_sequential_composition() {
        let mut sys: PetriNetSystem = MorphismSystem::new("pipeline".into());
        let net_a: PetriNet<char> = PetriNet::new(
            vec!['a', 'b'],
            vec![Transition::new(vec![(0, d(1))], vec![(1, d(1))])],
        );
        let net_b: PetriNet<char> = PetriNet::new(
            vec!['b', 'c'],
            vec![Transition::new(vec![(0, d(1))], vec![(1, d(1))])],
        );
        sys.add_definition_simple("first".into(), net_a).unwrap();
        sys.add_definition_simple("second".into(), net_b).unwrap();
        sys.add_definition_composite(
            "pipeline".into(),
            PetriNetDescription::Sequential("first".into(), "second".into()),
        ).unwrap();
        let resolved = sys.fill_black_boxes(None).unwrap();
        // Sequential merges 'b' sink of first with 'b' source of second
        assert_eq!(resolved.place_count(), 3);
        assert_eq!(resolved.transition_count(), 2);
    }

    #[test]
    fn morphism_system_nested_dag() {
        // Build: pipeline = sequential(parallel(A, B), C)
        let mut sys: PetriNetSystem = MorphismSystem::new("pipeline".into());

        let net_a: PetriNet<char> = PetriNet::new(
            vec!['a', 'x'],
            vec![Transition::new(vec![(0, d(1))], vec![(1, d(1))])],
        );
        let net_b: PetriNet<char> = PetriNet::new(
            vec!['b', 'y'],
            vec![Transition::new(vec![(0, d(1))], vec![(1, d(1))])],
        );
        let net_c: PetriNet<char> = PetriNet::new(
            vec!['x', 'z'],
            vec![Transition::new(vec![(0, d(1))], vec![(1, d(1))])],
        );

        sys.add_definition_simple("A".into(), net_a).unwrap();
        sys.add_definition_simple("B".into(), net_b).unwrap();
        sys.add_definition_simple("C".into(), net_c).unwrap();
        sys.add_definition_composite(
            "AB".into(),
            PetriNetDescription::Parallel("A".into(), "B".into()),
        ).unwrap();
        sys.add_definition_composite(
            "pipeline".into(),
            PetriNetDescription::Sequential("AB".into(), "C".into()),
        ).unwrap();

        let resolved = sys.fill_black_boxes(None).unwrap();
        // AB has 4 places [a,x,b,y], 2 transitions
        // C has 2 places [x,z], 1 transition
        // Sequential merges x (sink of AB) with x (source of C)
        assert_eq!(resolved.place_count(), 5); // a, x(merged), b, y, z
        assert_eq!(resolved.transition_count(), 3);
    }

    #[test]
    fn morphism_system_cycle_detected() {
        let mut sys: PetriNetSystem = MorphismSystem::new("A".into());
        sys.add_definition_composite(
            "A".into(),
            PetriNetDescription::Sequential("B".into(), "C".into()),
        ).unwrap();
        let result = sys.add_definition_composite(
            "B".into(),
            PetriNetDescription::Parallel("A".into(), "C".into()),
        );
        assert!(result.is_err());
    }

    #[test]
    fn morphism_system_black_box_resolves() {
        let mut sys: PetriNetSystem = MorphismSystem::new("top".into());
        sys.add_definition_simple("inner".into(), combustion_net()).unwrap();
        sys.add_definition_composite(
            "top".into(),
            PetriNetDescription::BlackBox("inner".into(), vec!['H', 'O'], vec!['W']),
        ).unwrap();
        let resolved = sys.fill_black_boxes(None).unwrap();
        assert_eq!(resolved.place_count(), 3);
        assert_eq!(resolved.transition_count(), 1);
    }
}
