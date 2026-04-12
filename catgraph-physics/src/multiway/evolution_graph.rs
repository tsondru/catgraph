//! Core data structure for multiway (non-deterministic) evolution.
//!
//! A multiway evolution graph represents branching computation where
//! multiple execution paths exist simultaneously. This captures:
//! - Non-deterministic Turing machines (multiple transitions per state)
//! - String rewriting systems (multiple rule applications)
//! - Hypergraph rewriting (Wolfram Physics model)
//!
//! ## Category Theory Connection
//!
//! In the category 𝒯 of computations with symmetric monoidal structure ⟨𝒯, ⊗, I⟩:
//! - Objects are states/configurations
//! - Morphisms are transitions
//! - Tensor product ⊗ represents parallel branches
//!
//! The functor Z': 𝒯 → ℬ maps this to the cobordism category, where
//! multicomputational irreducibility means Z' is a symmetric monoidal functor.

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::hash::{Hash, Hasher};


/// Unique identifier for a branch in the multiway graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BranchId(pub usize);

impl fmt::Display for BranchId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "B{}", self.0)
    }
}

/// Unique identifier for a node in the multiway graph.
///
/// Combines `branch_id` + step for globally unique identification.
/// This represents a specific state at a specific point in a specific branch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MultiwayNodeId {
    pub branch_id: BranchId,
    pub step: usize,
}

impl MultiwayNodeId {
    /// Create a new node ID.
    #[must_use]
    pub fn new(branch_id: BranchId, step: usize) -> Self {
        Self { branch_id, step }
    }
}

impl fmt::Display for MultiwayNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.branch_id, self.step)
    }
}

/// Edge type in the multiway graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MultiwayEdgeKind {
    /// Normal sequential transition within same branch.
    Sequential,
    /// Branch split: one parent, multiple children (fork/non-determinism).
    Fork {
        /// Index of the rule/transition chosen for this branch.
        rule_index: usize,
    },
    /// Branch merge: state reached from different paths (confluence).
    Merge,
}

/// An edge in the multiway evolution graph.
///
/// Connects a source node to a target node with a typed kind (sequential,
/// fork, or merge) and application-specific transition data `T` (e.g.,
/// which rewrite rule was applied and where).
#[derive(Clone, Debug)]
pub struct MultiwayEdge<T> {
    /// Source node ID.
    pub from: MultiwayNodeId,
    /// Target node ID.
    pub to: MultiwayNodeId,
    /// Type of edge (sequential, fork, or merge).
    pub kind: MultiwayEdgeKind,
    /// Application-specific transition data (rule applied, etc.).
    pub transition_data: T,
}

/// A node in the multiway evolution graph.
///
/// Stores the state `S` at a specific (branch, step) position, plus a
/// hash fingerprint for O(1) merge detection and cycle identification.
#[derive(Clone, Debug)]
pub struct MultiwayNode<S> {
    /// Unique identifier for this node.
    pub id: MultiwayNodeId,
    /// The state at this node.
    pub state: S,
    /// Hash fingerprint for fast equality checking and cycle detection.
    pub fingerprint: u64,
}

impl<S> MultiwayNode<S> {
    /// Create a new node.
    pub fn new(id: MultiwayNodeId, state: S, fingerprint: u64) -> Self {
        Self {
            id,
            state,
            fingerprint,
        }
    }
}

/// The multiway evolution graph representing branching computation.
///
/// This is the central data structure for analyzing multicomputational
/// irreducibility. It captures the full branching structure of a
/// non-deterministic computation.
///
/// ## Structure
///
/// - **Nodes**: States at (`branch_id`, step) positions
/// - **Edges**: Transitions including forks (branching) and merges (confluence)
/// - **Roots**: Initial states (typically one, but could have multiple)
///
/// ## Branching Semantics
///
/// When a state has multiple possible transitions:
/// 1. A **fork** is created with multiple outgoing edges
/// 2. Each edge leads to a new branch with a unique `BranchId`
/// 3. If two branches reach the same state, they can **merge**
#[derive(Clone, Debug)]
pub struct MultiwayEvolutionGraph<S, T> {
    /// All nodes indexed by their ID.
    nodes: HashMap<MultiwayNodeId, MultiwayNode<S>>,

    /// Forward edges: `from_id` -> list of edges.
    forward_edges: HashMap<MultiwayNodeId, Vec<MultiwayEdge<T>>>,

    /// Backward edges: `to_id` -> list of parent edges.
    backward_edges: HashMap<MultiwayNodeId, Vec<MultiwayEdge<T>>>,

    /// Root nodes (initial states).
    roots: Vec<MultiwayNodeId>,

    /// Next available branch ID.
    next_branch_id: usize,

    /// Maximum step reached across all branches.
    max_step: usize,

    /// Track active states by fingerprint for merge detection.
    /// Maps fingerprint -> canonical node ID for states at current frontier.
    active_states: HashMap<u64, MultiwayNodeId>,

    /// All leaf nodes (nodes with no outgoing edges).
    leaves: Vec<MultiwayNodeId>,

    /// Nodes indexed by their time step for O(1) lookup.
    step_nodes: HashMap<usize, Vec<MultiwayNodeId>>,
}

impl<S, T> Default for MultiwayEvolutionGraph<S, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, T> MultiwayEvolutionGraph<S, T> {
    /// Create a new empty multiway graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            forward_edges: HashMap::new(),
            backward_edges: HashMap::new(),
            roots: Vec::new(),
            next_branch_id: 0,
            max_step: 0,
            active_states: HashMap::new(),
            leaves: Vec::new(),
            step_nodes: HashMap::new(),
        }
    }

    /// Get the next branch ID and increment the counter.
    fn allocate_branch_id(&mut self) -> BranchId {
        let id = BranchId(self.next_branch_id);
        self.next_branch_id += 1;
        id
    }

    /// Get the number of nodes in the graph.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of edges in the graph.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.forward_edges.values().map(Vec::len).sum()
    }

    /// Get the maximum step reached.
    #[must_use]
    pub fn max_step(&self) -> usize {
        self.max_step
    }

    /// Get the number of branches created.
    #[must_use]
    pub fn branch_count(&self) -> usize {
        self.next_branch_id
    }

    /// Get root nodes.
    #[must_use]
    pub fn roots(&self) -> &[MultiwayNodeId] {
        &self.roots
    }

    /// Get leaf nodes.
    #[must_use]
    pub fn leaves(&self) -> &[MultiwayNodeId] {
        &self.leaves
    }

    /// Get a node by ID.
    #[must_use]
    pub fn get_node(&self, id: &MultiwayNodeId) -> Option<&MultiwayNode<S>> {
        self.nodes.get(id)
    }

    /// Get forward edges from a node.
    #[must_use]
    pub fn get_forward_edges(&self, id: &MultiwayNodeId) -> Option<&Vec<MultiwayEdge<T>>> {
        self.forward_edges.get(id)
    }

    /// Get backward edges to a node.
    #[must_use]
    pub fn get_backward_edges(&self, id: &MultiwayNodeId) -> Option<&Vec<MultiwayEdge<T>>> {
        self.backward_edges.get(id)
    }

    /// Check if a node is a fork point (has multiple outgoing edges).
    #[must_use]
    pub fn is_fork_point(&self, id: &MultiwayNodeId) -> bool {
        self.forward_edges
            .get(id)
            .is_some_and(|edges| edges.len() > 1)
    }

    /// Check if a node is a merge point (has multiple incoming edges).
    #[must_use]
    pub fn is_merge_point(&self, id: &MultiwayNodeId) -> bool {
        self.backward_edges
            .get(id)
            .is_some_and(|edges| edges.len() > 1)
    }
}

impl<S: Hash, T: Clone> MultiwayEvolutionGraph<S, T> {
    /// Compute fingerprint for a state.
    fn compute_fingerprint(state: &S) -> u64 {
        let mut hasher = DefaultHasher::new();
        state.hash(&mut hasher);
        hasher.finish()
    }

    /// Add a root node (initial state).
    ///
    /// Returns the ID of the created node.
    pub fn add_root(&mut self, state: S) -> MultiwayNodeId {
        let branch_id = self.allocate_branch_id();
        let id = MultiwayNodeId::new(branch_id, 0);
        let fingerprint = Self::compute_fingerprint(&state);

        let node = MultiwayNode::new(id, state, fingerprint);
        self.nodes.insert(id, node);
        self.step_nodes.entry(0).or_default().push(id);
        self.roots.push(id);
        self.leaves.push(id);
        self.active_states.insert(fingerprint, id);

        id
    }

    /// Add a sequential edge (non-branching step).
    ///
    /// Creates a new node in the same branch at step + 1.
    /// Returns the ID of the new node.
    pub fn add_sequential_step(
        &mut self,
        from: MultiwayNodeId,
        state: S,
        transition_data: T,
    ) -> MultiwayNodeId {
        let new_step = from.step + 1;
        let id = MultiwayNodeId::new(from.branch_id, new_step);
        let fingerprint = Self::compute_fingerprint(&state);

        // Create node
        let node = MultiwayNode::new(id, state, fingerprint);
        self.nodes.insert(id, node);
        self.step_nodes.entry(new_step).or_default().push(id);

        // Create edge
        let edge = MultiwayEdge {
            from,
            to: id,
            kind: MultiwayEdgeKind::Sequential,
            transition_data,
        };

        self.forward_edges.entry(from).or_default().push(edge.clone());
        self.backward_edges.entry(id).or_default().push(edge);

        // Update tracking
        self.max_step = self.max_step.max(new_step);
        self.leaves.retain(|&leaf| leaf != from);
        self.leaves.push(id);

        // Update active states (remove old, add new)
        if let Some(old_node) = self.nodes.get(&from) {
            self.active_states.remove(&old_node.fingerprint);
        }
        self.active_states.insert(fingerprint, id);

        id
    }

    /// Add a fork (one parent, multiple children from non-determinism).
    ///
    /// Each branch gets a new `BranchId`. Returns Vec of new node IDs.
    ///
    /// # Arguments
    /// * `from` - The parent node ID
    /// * `branches` - Vec of (state, `transition_data`, `rule_index`) for each branch
    pub fn add_fork(
        &mut self,
        from: MultiwayNodeId,
        branches: Vec<(S, T, usize)>,
    ) -> Vec<MultiwayNodeId> {
        let new_step = from.step + 1;
        let mut new_ids = Vec::with_capacity(branches.len());

        // Remove parent from leaves
        self.leaves.retain(|&leaf| leaf != from);

        // Remove parent from active states
        if let Some(old_node) = self.nodes.get(&from) {
            self.active_states.remove(&old_node.fingerprint);
        }

        for (state, transition_data, rule_index) in branches {
            let fingerprint = Self::compute_fingerprint(&state);

            // Check for merge: does this state already exist at this step?
            // For simplicity, we still create the node but could optimize later
            let branch_id = self.allocate_branch_id();
            let id = MultiwayNodeId::new(branch_id, new_step);

            // Create node
            let node = MultiwayNode::new(id, state, fingerprint);
            self.nodes.insert(id, node);
            self.step_nodes.entry(new_step).or_default().push(id);

            // Create edge
            let edge = MultiwayEdge {
                from,
                to: id,
                kind: MultiwayEdgeKind::Fork { rule_index },
                transition_data,
            };

            self.forward_edges.entry(from).or_default().push(edge.clone());
            self.backward_edges.entry(id).or_default().push(edge);

            // Update tracking
            self.leaves.push(id);
            self.active_states.insert(fingerprint, id);
            new_ids.push(id);
        }

        self.max_step = self.max_step.max(new_step);
        new_ids
    }

    /// Try to find an existing node with the same fingerprint at the current frontier.
    ///
    /// Returns the canonical node ID if a merge is possible.
    #[must_use]
    pub fn find_merge_candidate(&self, fingerprint: u64) -> Option<MultiwayNodeId> {
        self.active_states.get(&fingerprint).copied()
    }

    /// Add a merge edge from a node to an existing canonical node.
    ///
    /// This represents confluence: two branches reaching the same state.
    pub fn add_merge_edge(&mut self, from: MultiwayNodeId, to: MultiwayNodeId, transition_data: T) {
        let edge = MultiwayEdge {
            from,
            to,
            kind: MultiwayEdgeKind::Merge,
            transition_data,
        };

        self.forward_edges.entry(from).or_default().push(edge.clone());
        self.backward_edges.entry(to).or_default().push(edge);

        // from is no longer a leaf (it transitions to existing node)
        self.leaves.retain(|&leaf| leaf != from);
    }

    /// Get all nodes at a specific time step (branchlike hypersurface `Σ_t`).
    ///
    /// Uses the pre-built step index for O(1) lookup instead of scanning all nodes.
    #[must_use]
    pub fn nodes_at_step(&self, step: usize) -> Vec<&MultiwayNode<S>> {
        self.step_nodes
            .get(&step)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all node IDs at a specific step.
    ///
    /// Uses the pre-built step index for O(1) lookup instead of scanning all nodes.
    #[must_use]
    pub fn node_ids_at_step(&self, step: usize) -> Vec<MultiwayNodeId> {
        self.step_nodes.get(&step).cloned().unwrap_or_default()
    }

    /// Find all fork points in the graph.
    #[must_use]
    pub fn find_fork_points(&self) -> Vec<MultiwayNodeId> {
        self.nodes
            .keys()
            .filter(|id| self.is_fork_point(id))
            .copied()
            .collect()
    }

    /// Find all merge points in the graph.
    #[must_use]
    pub fn find_merge_points(&self) -> Vec<MultiwayNodeId> {
        self.nodes
            .keys()
            .filter(|id| self.is_merge_point(id))
            .copied()
            .collect()
    }

    /// Find cycles across branches (same fingerprint at different nodes).
    #[must_use]
    pub fn find_cycles_across_branches(&self) -> Vec<MultiwayCycle> {
        let mut fingerprint_occurrences: HashMap<u64, Vec<MultiwayNodeId>> = HashMap::new();

        for node in self.nodes.values() {
            fingerprint_occurrences
                .entry(node.fingerprint)
                .or_default()
                .push(node.id);
        }

        let mut cycles = Vec::new();
        for (fingerprint, occurrences) in fingerprint_occurrences {
            if occurrences.len() > 1 {
                // Sort by step for consistent ordering
                let mut sorted = occurrences;
                sorted.sort_by_key(|id| (id.step, id.branch_id.0));

                for i in 0..sorted.len() - 1 {
                    cycles.push(MultiwayCycle {
                        first_occurrence: sorted[i],
                        second_occurrence: sorted[i + 1],
                        fingerprint,
                    });
                }
            }
        }

        cycles
    }

    /// Trace path from a node back to its root.
    ///
    /// Public so downstream crates (e.g. `irreducible`) can reimplement
    /// interval-bridge helpers without needing the private fields of
    /// `MultiwayEvolutionGraph`.
    #[must_use]
    pub fn trace_path_to_root(&self, from: MultiwayNodeId) -> Vec<MultiwayNodeId> {
        let mut path = vec![from];
        let mut current = from;

        while let Some(edges) = self.backward_edges.get(&current) {
            if let Some(edge) = edges.first() {
                path.push(edge.from);
                current = edge.from;
            } else {
                break;
            }
        }

        path.reverse();
        path
    }

    /// Get statistics about the multiway graph.
    #[must_use]
    pub fn statistics(&self) -> MultiwayStatistics {
        let fork_points = self.find_fork_points();
        let merge_points = self.find_merge_points();

        MultiwayStatistics {
            total_nodes: self.nodes.len(),
            total_edges: self.edge_count(),
            max_branches: self.next_branch_id,
            max_depth: self.max_step,
            merge_count: merge_points.len(),
            fork_count: fork_points.len(),
            leaf_count: self.leaves.len(),
            root_count: self.roots.len(),
        }
    }
}

/// A cycle detected across branches.
///
/// Represents the same state appearing at different points in the
/// multiway evolution, which may indicate reducibility.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultiwayCycle {
    /// First occurrence of the repeated state.
    pub first_occurrence: MultiwayNodeId,
    /// Second occurrence of the repeated state.
    pub second_occurrence: MultiwayNodeId,
    /// Hash fingerprint of the repeated state.
    pub fingerprint: u64,
}

impl MultiwayCycle {
    /// The step difference between occurrences.
    #[must_use]
    pub fn step_difference(&self) -> usize {
        self.second_occurrence
            .step
            .saturating_sub(self.first_occurrence.step)
    }

    /// Whether the cycle is within the same branch.
    #[must_use]
    pub fn is_same_branch(&self) -> bool {
        self.first_occurrence.branch_id == self.second_occurrence.branch_id
    }
}

/// Aggregate statistics about a multiway evolution graph.
///
/// Summarizes the graph topology: total nodes/edges, branching depth,
/// and counts of fork points (non-deterministic splits) and merge points
/// (confluences where distinct branches reach the same state).
#[derive(Clone, Debug, Default)]
pub struct MultiwayStatistics {
    /// Total number of nodes.
    pub total_nodes: usize,
    /// Total number of edges.
    pub total_edges: usize,
    /// Maximum number of branches created.
    pub max_branches: usize,
    /// Maximum depth (step) reached.
    pub max_depth: usize,
    /// Number of merge points (confluence).
    pub merge_count: usize,
    /// Number of fork points (branching).
    pub fork_count: usize,
    /// Number of leaf nodes.
    pub leaf_count: usize,
    /// Number of root nodes.
    pub root_count: usize,
}

impl fmt::Display for MultiwayStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MultiwayStats {{ nodes: {}, edges: {}, branches: {}, depth: {}, forks: {}, merges: {} }}",
            self.total_nodes,
            self.total_edges,
            self.max_branches,
            self.max_depth,
            self.fork_count,
            self.merge_count
        )
    }
}

/// A merge point where distinct branches converge to the same state.
///
/// Represents confluence in the multiway graph: the `merged_node` has
/// multiple incoming edges from different `parent_nodes` on separate branches.
#[derive(Clone, Debug)]
pub struct MergePoint {
    /// The node where branches merge.
    pub merged_node: MultiwayNodeId,
    /// Parent nodes from different branches.
    pub parent_nodes: Vec<MultiwayNodeId>,
}

/// Run multiway BFS evolution with a domain-specific step function.
///
/// Generic BFS loop shared by all multiway systems. The `step_fn` takes
/// a state reference and returns all possible successor states with
/// their transition data and a rule index label.
///
/// # Arguments
/// * `initial` - The initial state
/// * `step_fn` - Closure that computes all successors: `&S -> Vec<(S, T, usize)>`
/// * `max_steps` - Maximum BFS depth per branch
/// * `max_branches` - Maximum total branches to explore
///
/// # Algorithm (pure BFS, follows the NTM pattern)
///
/// 1. Create graph, add root
/// 2. frontier = `VecDeque` with (`root_id`, `initial_state`)
/// 3. Pop from frontier; skip if step >= `max_steps`; break if budget exhausted
/// 4. Single successor  → sequential step
/// 5. Multiple successors → fork (capped by remaining branch budget)
///
/// # Panics
///
/// Panics if the graph node lookup fails for a node that was just added.
pub fn run_multiway_bfs<S, T, F>(
    initial: S,
    step_fn: F,
    max_steps: usize,
    max_branches: usize,
) -> MultiwayEvolutionGraph<S, T>
where
    S: Clone + Hash,
    T: Clone,
    F: Fn(&S) -> Vec<(S, T, usize)>,
{
    let mut graph = MultiwayEvolutionGraph::new();
    let root_id = graph.add_root(initial.clone());

    let mut frontier: VecDeque<(MultiwayNodeId, S)> = VecDeque::new();
    frontier.push_back((root_id, initial));

    let mut total_branches: usize = 1;

    while let Some((node_id, state)) = frontier.pop_front() {
        // Check step limit
        if node_id.step >= max_steps {
            continue;
        }

        // Check branch limit
        if total_branches >= max_branches {
            break;
        }

        let next_steps = step_fn(&state);

        if next_steps.is_empty() {
            // Terminal state — no applicable transitions
            continue;
        }

        if next_steps.len() == 1 {
            // Deterministic step: sequential
            let (new_state, transition_data, _rule_index) =
                next_steps.into_iter().next().unwrap();
            let new_id =
                graph.add_sequential_step(node_id, new_state.clone(), transition_data);
            frontier.push_back((new_id, new_state));
        } else {
            // Non-deterministic: fork
            let branches_to_add =
                next_steps.len().min(max_branches - total_branches + 1);

            let fork_data: Vec<(S, T, usize)> =
                next_steps.into_iter().take(branches_to_add).collect();

            let new_ids = graph.add_fork(node_id, fork_data.clone());
            total_branches += new_ids.len().saturating_sub(1);

            for (id, (new_state, _, _)) in new_ids.iter().zip(fork_data.iter()) {
                frontier.push_back((*id, new_state.clone()));
            }
        }
    }

    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_root_creates_node() {
        let mut graph: MultiwayEvolutionGraph<String, ()> = MultiwayEvolutionGraph::new();
        let id = graph.add_root("initial".to_string());

        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.roots().len(), 1);
        assert_eq!(id.step, 0);
        assert_eq!(id.branch_id, BranchId(0));
    }

    #[test]
    fn test_sequential_edges_form_chain() {
        let mut graph: MultiwayEvolutionGraph<String, &str> = MultiwayEvolutionGraph::new();
        let root = graph.add_root("A".to_string());
        let n1 = graph.add_sequential_step(root, "B".to_string(), "A->B");
        let n2 = graph.add_sequential_step(n1, "C".to_string(), "B->C");

        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2);
        assert_eq!(n2.step, 2);
        assert_eq!(n2.branch_id, BranchId(0)); // Same branch
    }

    #[test]
    fn test_fork_creates_multiple_branches() {
        let mut graph: MultiwayEvolutionGraph<String, usize> = MultiwayEvolutionGraph::new();
        let root = graph.add_root("start".to_string());

        let fork_branches = vec![
            ("A".to_string(), 0, 0),
            ("B".to_string(), 1, 1),
            ("C".to_string(), 2, 2),
        ];
        let new_ids = graph.add_fork(root, fork_branches);

        assert_eq!(new_ids.len(), 3);
        assert_eq!(graph.node_count(), 4); // root + 3 branches
        assert_eq!(graph.edge_count(), 3);
        assert!(graph.is_fork_point(&root));

        // Each branch has unique ID
        let branch_ids: Vec<_> = new_ids.iter().map(|id| id.branch_id).collect();
        assert_eq!(branch_ids.len(), 3);
        assert!(branch_ids.iter().all(|&b| b != root.branch_id));
    }

    #[test]
    fn test_nodes_at_step_returns_correct_slice() {
        let mut graph: MultiwayEvolutionGraph<i32, ()> = MultiwayEvolutionGraph::new();
        let root = graph.add_root(0);
        let fork_branches = vec![(1, (), 0), (2, (), 1)];
        graph.add_fork(root, fork_branches);

        let step0 = graph.nodes_at_step(0);
        let step1 = graph.nodes_at_step(1);

        assert_eq!(step0.len(), 1);
        assert_eq!(step1.len(), 2);
    }

    #[test]
    fn test_find_cycles_across_branches() {
        let mut graph: MultiwayEvolutionGraph<i32, ()> = MultiwayEvolutionGraph::new();
        let root = graph.add_root(42);

        // Create two branches that reach the same state
        let branches = vec![(1, (), 0), (2, (), 1)];
        let ids = graph.add_fork(root, branches);

        // Both branches step to the same value (42 again)
        graph.add_sequential_step(ids[0], 42, ());
        graph.add_sequential_step(ids[1], 42, ());

        let cycles = graph.find_cycles_across_branches();
        // Should find cycles: root (42) matches later occurrences
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_trace_path_to_root_linear_chain() {
        // Regression test replacing the old to_branch_intervals test.
        // trace_path_to_root is now the public primitive downstream crates
        // build interval/step bridges on top of.
        let mut graph: MultiwayEvolutionGraph<char, ()> = MultiwayEvolutionGraph::new();
        let root = graph.add_root('A');
        let n1 = graph.add_sequential_step(root, 'B', ());
        let n2 = graph.add_sequential_step(n1, 'C', ());

        let path = graph.trace_path_to_root(n2);
        assert_eq!(path.len(), 3, "A -> B -> C is length-3 path");
        assert_eq!(path[0], root, "path starts at root");
        assert_eq!(path[2], n2, "path ends at leaf");
        // Steps progress 0, 1, 2 along a sequential chain.
        assert_eq!(path[0].step, 0);
        assert_eq!(path[1].step, 1);
        assert_eq!(path[2].step, 2);
    }

    #[test]
    fn test_statistics() {
        let mut graph: MultiwayEvolutionGraph<i32, ()> = MultiwayEvolutionGraph::new();
        let root = graph.add_root(0);
        let branches = vec![(1, (), 0), (2, (), 1)];
        graph.add_fork(root, branches);

        let stats = graph.statistics();
        assert_eq!(stats.total_nodes, 3);
        assert_eq!(stats.total_edges, 2);
        assert_eq!(stats.fork_count, 1);
        assert_eq!(stats.leaf_count, 2);
    }

    #[test]
    fn test_branch_id_display() {
        let id = BranchId(42);
        assert_eq!(format!("{}", id), "B42");
    }

    #[test]
    fn test_node_id_display() {
        let id = MultiwayNodeId::new(BranchId(3), 7);
        assert_eq!(format!("{}", id), "B3@7");
    }
}
