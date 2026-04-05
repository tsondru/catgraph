//! Hypergraph evolution tracking and causal invariance analysis.
//!
//! This module tracks the history of hypergraph rewrites and provides
//! tools for analyzing causal invariance via Wilson loops.

use super::hypergraph::Hypergraph;
use super::rewrite_rule::{RewriteMatch, RewriteRule, RewriteSpan};
use std::collections::{BTreeSet, HashMap, HashSet};

use crate::category::Composable;
use crate::cospan::Cospan;
use crate::span::Span;

/// A single step in the multiway evolution of a hypergraph.
///
/// Records which rule was applied, the match site, the resulting
/// hypergraph state (with fingerprint), and the parent step for
/// tree traversal.
#[derive(Debug, Clone)]
pub struct HypergraphStep {
    /// The rule that was applied.
    pub rule_index: usize,

    /// The match where the rule was applied.
    pub match_info: RewriteMatch,

    /// State of the hypergraph after this step.
    pub state: Hypergraph,

    /// Fingerprint of the state (for fast comparison).
    pub fingerprint: u64,

    /// Step number (0-indexed).
    pub step: usize,

    /// Parent step index (None for initial state).
    pub parent: Option<usize>,

    /// Branch ID (for multiway evolution).
    pub branch_id: usize,
}

/// A node in the hypergraph evolution graph (multiway systems).
///
/// Stores the full hypergraph state at a given depth, with a fingerprint
/// for fast equality checks and optional parent/transition provenance.
#[derive(Debug, Clone)]
pub struct HypergraphNode {
    /// Unique ID for this node.
    pub id: usize,

    /// The hypergraph state at this node.
    pub state: Hypergraph,

    /// Fingerprint for fast comparison.
    pub fingerprint: u64,

    /// Step (depth) in the evolution.
    pub step: usize,

    /// Parent node ID (None for root).
    pub parent: Option<usize>,

    /// Rule and match that led to this state (None for root).
    pub transition: Option<(usize, RewriteMatch)>,
}

/// A Wilson loop in the hypergraph evolution history.
///
/// A closed path in the rewrite history graph, analogous to a Wilson loop
/// in lattice gauge theory. The holonomy (product of transformations
/// around the loop) measures deviation from path-independence:
/// holonomy = 1.0 means the system is causally invariant along this loop.
#[derive(Debug, Clone)]
pub struct WilsonLoop {
    /// Sequence of node IDs forming the loop.
    pub path: Vec<usize>,

    /// Starting/ending node ID.
    pub base: usize,

    /// Holonomy value (1.0 = perfect closure, causally invariant).
    pub holonomy: f64,

    /// Length of the loop.
    pub length: usize,
}

/// Result of causal invariance analysis.
#[derive(Debug, Clone)]
pub struct CausalInvarianceResult {
    /// Whether the system is causally invariant.
    pub is_invariant: bool,

    /// Average holonomy deviation from 1.0.
    pub average_deviation: f64,

    /// Maximum holonomy deviation from 1.0.
    pub max_deviation: f64,

    /// Number of Wilson loops analyzed.
    pub loops_analyzed: usize,

    /// Wilson loops with significant deviation.
    pub non_trivial_loops: Vec<WilsonLoop>,
}

/// Evolution of a hypergraph under rewrite rules.
///
/// Tracks the history of rewrites and supports both deterministic
/// (single path) and non-deterministic (multiway) evolution.
#[derive(Debug, Clone)]
pub struct HypergraphEvolution {
    /// All nodes in the evolution graph.
    nodes: Vec<HypergraphNode>,

    /// Rules used in this evolution.
    rules: Vec<RewriteRule>,

    /// Map from fingerprint to node IDs (for detecting merges).
    fingerprint_to_nodes: HashMap<u64, Vec<usize>>,

    /// Maximum step reached.
    max_step: usize,

    /// Next vertex ID for new vertices.
    next_vertex_id: usize,
}

impl HypergraphEvolution {
    /// Creates a new evolution starting from the given hypergraph.
    #[must_use]
    pub fn new(initial: Hypergraph, rules: Vec<RewriteRule>) -> Self {
        let fingerprint = initial.fingerprint();
        let max_vertex = initial.vertices().max().unwrap_or(0);

        let root = HypergraphNode {
            id: 0,
            state: initial,
            fingerprint,
            step: 0,
            parent: None,
            transition: None,
        };

        let mut fingerprint_to_nodes = HashMap::new();
        fingerprint_to_nodes.insert(fingerprint, vec![0]);

        Self {
            nodes: vec![root],
            rules,
            fingerprint_to_nodes,
            max_step: 0,
            next_vertex_id: max_vertex + 1,
        }
    }

    /// Runs deterministic evolution for the given number of steps.
    ///
    /// At each step, applies the first matching rule at the first match.
    ///
    /// # Arguments
    ///
    /// * `initial` - Starting hypergraph
    /// * `rules` - Rewrite rules to apply
    /// * `max_steps` - Maximum number of rewrite steps
    ///
    /// # Returns
    ///
    /// An evolution with the deterministic trace.
    #[must_use]
    pub fn run(initial: &Hypergraph, rules: &[RewriteRule], max_steps: usize) -> Self {
        let mut evolution = Self::new(initial.clone(), rules.to_vec());
        let mut current_id = 0;

        for _ in 0..max_steps {
            let node = &evolution.nodes[current_id];
            let state = node.state.clone();

            // Find first applicable rule
            let mut applied = false;
            for (rule_idx, rule) in rules.iter().enumerate() {
                let matches = rule.find_matches(&state);
                if !matches.is_empty() {
                    // Apply first match
                    let new_id = evolution.apply_rule(current_id, rule_idx, &matches[0]);
                    current_id = new_id;
                    applied = true;
                    break;
                }
            }

            if !applied {
                break; // No rules apply
            }
        }

        evolution
    }

    /// Runs multiway (non-deterministic) evolution.
    ///
    /// Explores all possible rule applications up to limits.
    ///
    /// # Arguments
    ///
    /// * `initial` - Starting hypergraph
    /// * `rules` - Rewrite rules to apply
    /// * `max_steps` - Maximum depth
    /// * `max_nodes` - Maximum total nodes to explore
    ///
    /// # Returns
    ///
    /// An evolution with the multiway graph.
    #[must_use]
    pub fn run_multiway(
        initial: &Hypergraph,
        rules: &[RewriteRule],
        max_steps: usize,
        max_nodes: usize,
    ) -> Self {
        let mut evolution = Self::new(initial.clone(), rules.to_vec());
        let mut frontier = vec![0usize]; // Nodes to expand

        while !frontier.is_empty() && evolution.nodes.len() < max_nodes {
            let current_id = frontier.remove(0);
            let node = &evolution.nodes[current_id];

            if node.step >= max_steps {
                continue;
            }

            let state = node.state.clone();

            // Find all applicable rules and matches
            for (rule_idx, rule) in rules.iter().enumerate() {
                let matches = rule.find_matches(&state);
                for match_ in matches {
                    if evolution.nodes.len() >= max_nodes {
                        break;
                    }
                    let new_id = evolution.apply_rule(current_id, rule_idx, &match_);
                    frontier.push(new_id);
                }
            }
        }

        evolution
    }

    /// Applies a rule at a specific node and match.
    ///
    /// # Returns
    ///
    /// The ID of the newly created node.
    fn apply_rule(
        &mut self,
        parent_id: usize,
        rule_idx: usize,
        match_: &RewriteMatch,
    ) -> usize {
        let parent = &self.nodes[parent_id];
        let mut new_state = parent.state.clone();
        let parent_step = parent.step;

        // Apply the rule
        let rule = &self.rules[rule_idx];
        rule.apply(&mut new_state, match_, &mut self.next_vertex_id);

        let fingerprint = new_state.fingerprint();
        let new_id = self.nodes.len();
        let new_step = parent_step + 1;

        let node = HypergraphNode {
            id: new_id,
            state: new_state,
            fingerprint,
            step: new_step,
            parent: Some(parent_id),
            transition: Some((rule_idx, match_.clone())),
        };

        self.nodes.push(node);
        self.fingerprint_to_nodes
            .entry(fingerprint)
            .or_default()
            .push(new_id);
        self.max_step = self.max_step.max(new_step);

        new_id
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Returns the number of nodes in the evolution.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the maximum step reached.
    #[must_use]
    pub fn max_step(&self) -> usize {
        self.max_step
    }

    /// Returns a reference to a node by ID.
    #[must_use]
    pub fn get_node(&self, id: usize) -> Option<&HypergraphNode> {
        self.nodes.get(id)
    }

    /// Returns the root (initial) node.
    #[must_use]
    pub fn root(&self) -> &HypergraphNode {
        &self.nodes[0]
    }

    /// Returns all leaf nodes (nodes with no children).
    #[must_use]
    pub fn leaves(&self) -> Vec<usize> {
        let parents: HashSet<_> = self
            .nodes
            .iter()
            .filter_map(|n| n.parent)
            .collect();

        (0..self.nodes.len())
            .filter(|id| !parents.contains(id))
            .collect()
    }

    /// Returns nodes at a specific step.
    #[must_use]
    pub fn nodes_at_step(&self, step: usize) -> Vec<usize> {
        self.nodes
            .iter()
            .filter(|n| n.step == step)
            .map(|n| n.id)
            .collect()
    }

    /// Finds merge points (nodes with same fingerprint from different parents).
    #[must_use]
    pub fn find_merges(&self) -> Vec<Vec<usize>> {
        self.fingerprint_to_nodes
            .values()
            .filter(|ids| ids.len() > 1)
            .cloned()
            .collect()
    }

    // ========================================================================
    // Causal Invariance Analysis
    // ========================================================================

    /// Finds all Wilson loops (closed paths) in the evolution graph.
    ///
    /// A Wilson loop exists when two different paths from the root
    /// lead to isomorphic hypergraph states.
    #[must_use]
    pub fn find_wilson_loops(&self) -> Vec<WilsonLoop> {
        let mut loops = Vec::new();

        // Find merge points (same fingerprint from different paths)
        for ids in self.fingerprint_to_nodes.values() {
            if ids.len() < 2 {
                continue;
            }

            // For each pair of nodes with same fingerprint
            for i in 0..ids.len() {
                for j in (i + 1)..ids.len() {
                    let id1 = ids[i];
                    let id2 = ids[j];

                    // Check if they're actually isomorphic (not just same fingerprint)
                    let n1 = &self.nodes[id1];
                    let n2 = &self.nodes[id2];

                    if n1.state.is_isomorphic_to(&n2.state) {
                        // Found a Wilson loop
                        let path1 = self.path_to_root(id1);
                        let path2 = self.path_to_root(id2);

                        // Find common ancestor
                        let path1_set: HashSet<_> = path1.iter().copied().collect();
                        let ancestor = path2
                            .iter()
                            .find(|id| path1_set.contains(id))
                            .copied()
                            .unwrap_or(0);

                        // Build the loop path
                        let mut loop_path = Vec::new();

                        // Path from ancestor to id1
                        for &id in path1.iter().rev() {
                            loop_path.push(id);
                            if id == ancestor {
                                break;
                            }
                        }

                        // Path from id2 back to ancestor
                        let mut path2_segment = Vec::new();
                        for &id in &path2 {
                            if id == ancestor {
                                break;
                            }
                            path2_segment.push(id);
                        }
                        path2_segment.reverse();
                        loop_path.extend(path2_segment);

                        // Compute holonomy
                        let holonomy = self.compute_holonomy(&loop_path);

                        loops.push(WilsonLoop {
                            path: loop_path.clone(),
                            base: ancestor,
                            holonomy,
                            length: loop_path.len(),
                        });
                    }
                }
            }
        }

        loops
    }

    /// Returns the path from a node to the root.
    fn path_to_root(&self, node_id: usize) -> Vec<usize> {
        let mut path = vec![node_id];
        let mut current = node_id;

        while let Some(parent) = self.nodes[current].parent {
            path.push(parent);
            current = parent;
        }

        path
    }

    /// Computes the holonomy of a loop.
    ///
    /// Holonomy measures how much the state changes when going around a loop.
    /// - Holonomy = 1.0: Perfect closure (causally invariant)
    /// - Holonomy < 1.0: State differs after traversing the loop
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn compute_holonomy(&self, loop_path: &[usize]) -> f64 {
        if loop_path.len() < 2 {
            return 1.0;
        }

        let start_node = &self.nodes[loop_path[0]];
        let end_node = &self.nodes[*loop_path.last().unwrap()];

        // Compare states using isomorphism check
        if start_node.state.is_isomorphic_to(&end_node.state) {
            1.0
        } else {
            // Compute similarity based on structural overlap
            let start_edges = start_node.state.edge_count();
            let end_edges = end_node.state.edge_count();

            if start_edges == 0 && end_edges == 0 {
                return 1.0;
            }

            // Simple similarity measure
            let common_vertices = start_node
                .state
                .vertices()
                .filter(|v| end_node.state.contains_vertex(*v))
                .count();
            let total_vertices =
                start_node.state.vertex_count().max(end_node.state.vertex_count());

            if total_vertices == 0 {
                1.0
            } else {
                common_vertices as f64 / total_vertices as f64
            }
        }
    }

    /// Analyzes causal invariance of the evolution.
    ///
    /// A system is causally invariant if all Wilson loops have holonomy = 1.0,
    /// meaning the final state is independent of the order of rule applications.
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn analyze_causal_invariance(&self) -> CausalInvarianceResult {
        let loops = self.find_wilson_loops();

        if loops.is_empty() {
            return CausalInvarianceResult {
                is_invariant: true, // No loops = trivially invariant
                average_deviation: 0.0,
                max_deviation: 0.0,
                loops_analyzed: 0,
                non_trivial_loops: vec![],
            };
        }

        let deviations: Vec<_> = loops.iter().map(|l| (1.0 - l.holonomy).abs()).collect();

        let average_deviation = deviations.iter().sum::<f64>() / deviations.len() as f64;
        let max_deviation = deviations.iter().copied().fold(0.0, f64::max);

        // Consider loops with deviation > 0.01 as non-trivial
        let non_trivial_loops: Vec<_> = loops
            .into_iter()
            .filter(|l| (1.0 - l.holonomy).abs() > 0.01)
            .collect();

        let is_invariant = max_deviation < 0.01;

        CausalInvarianceResult {
            is_invariant,
            average_deviation,
            max_deviation,
            loops_analyzed: deviations.len(),
            non_trivial_loops,
        }
    }

    /// Checks if the system is causally invariant.
    ///
    /// This is a quick check that returns true if all explored paths
    /// that lead to isomorphic states have holonomy ≈ 1.0.
    #[must_use]
    pub fn is_causally_invariant(&self) -> bool {
        self.analyze_causal_invariance().is_invariant
    }

    // ========================================================================
    // Statistics
    // ========================================================================

    /// Returns statistics about the evolution.
    #[must_use]
    pub fn statistics(&self) -> EvolutionStatistics {
        let leaves = self.leaves();
        let merges = self.find_merges();

        let branch_count = leaves.len();
        let merge_count = merges.len();

        // Count rule applications
        let mut rule_counts = vec![0; self.rules.len()];
        for node in &self.nodes {
            if let Some((rule_idx, _)) = &node.transition {
                rule_counts[*rule_idx] += 1;
            }
        }

        EvolutionStatistics {
            total_nodes: self.nodes.len(),
            max_step: self.max_step,
            branch_count,
            merge_count,
            rule_applications: rule_counts,
        }
    }
}

/// Statistics about a hypergraph evolution.
#[derive(Debug, Clone)]
pub struct EvolutionStatistics {
    /// Total number of nodes explored.
    pub total_nodes: usize,

    /// Maximum depth reached.
    pub max_step: usize,

    /// Number of distinct branches (leaf nodes).
    pub branch_count: usize,

    /// Number of merge points (confluence).
    pub merge_count: usize,

    /// Number of times each rule was applied.
    pub rule_applications: Vec<usize>,
}

impl std::fmt::Display for EvolutionStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Evolution Statistics:")?;
        writeln!(f, "  Total nodes: {}", self.total_nodes)?;
        writeln!(f, "  Max step: {}", self.max_step)?;
        writeln!(f, "  Branches: {}", self.branch_count)?;
        writeln!(f, "  Merges: {}", self.merge_count)?;
        for (i, count) in self.rule_applications.iter().enumerate() {
            writeln!(f, "  Rule {i}: {count} applications")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for CausalInvarianceResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Causal Invariance Analysis:")?;
        writeln!(
            f,
            "  Causally invariant: {}",
            if self.is_invariant { "YES" } else { "NO" }
        )?;
        writeln!(f, "  Loops analyzed: {}", self.loops_analyzed)?;
        writeln!(f, "  Average deviation: {:.6}", self.average_deviation)?;
        writeln!(f, "  Max deviation: {:.6}", self.max_deviation)?;
        writeln!(f, "  Non-trivial loops: {}", self.non_trivial_loops.len())?;
        Ok(())
    }
}

// ============================================================================
// RewriteRule → Span
// ============================================================================

impl RewriteRule {
    /// Converts this rewrite rule to its categorical span representation.
    ///
    /// A rewrite rule L → R with shared variables K is naturally a span:
    ///
    /// ```text
    ///     L ←── K ──→ R
    /// ```
    ///
    /// - L elements = unique variables in the left pattern
    /// - R elements = unique variables in the right pattern
    /// - K elements = preserved variables (appear in both L and R)
    /// - Each K element maps to its index in L and its index in R
    ///
    /// Labels are `u32` variable IDs, so the span carries which variables
    /// are on each side (e.g., `left() = [0, 1, 2]` for variables 0, 1, 2).
    ///
    /// # Example
    ///
    /// ```rust
    /// use catgraph::hypergraph::RewriteRule;
    ///
    /// // Wolfram A→BB: {0,1,2} → {0,1},{1,2}
    /// let rule = RewriteRule::wolfram_a_to_bb();
    /// let span = rule.to_span();
    ///
    /// // L has 3 variables (0,1,2), R has 3 variables (0,1,2)
    /// assert_eq!(span.left(), &[0u32, 1, 2]);
    /// assert_eq!(span.right(), &[0u32, 1, 2]);
    /// // K = {0,1,2} (all preserved) → 3 middle pairs
    /// assert_eq!(span.middle_pairs().len(), 3);
    /// ```
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    #[must_use]
    pub fn to_span(&self) -> Span<u32> {
        // Collect unique variables from each side, sorted for determinism
        let left_vars: BTreeSet<usize> = self.left().iter()
            .flat_map(|e| e.vertices().iter().copied())
            .collect();
        let right_vars: BTreeSet<usize> = self.right().iter()
            .flat_map(|e| e.vertices().iter().copied())
            .collect();

        let left_sorted: Vec<usize> = left_vars.iter().copied().collect();
        let right_sorted: Vec<usize> = right_vars.iter().copied().collect();

        // Build index maps: variable → position in sorted vec
        let left_index: HashMap<usize, usize> = left_sorted.iter()
            .enumerate()
            .map(|(i, &v)| (v, i))
            .collect();
        let right_index: HashMap<usize, usize> = right_sorted.iter()
            .enumerate()
            .map(|(i, &v)| (v, i))
            .collect();

        // Kernel = preserved variables (in both L and R)
        let preserved = self.preserved_variables();
        let mut middle: Vec<(usize, usize)> = preserved.iter()
            .map(|&v| (left_index[&v], right_index[&v]))
            .collect();
        // Sort for deterministic output
        middle.sort_unstable();

        // Labels are variable IDs (as u32)
        let left_labels: Vec<u32> = left_sorted.iter().map(|&v| v as u32).collect();
        let right_labels: Vec<u32> = right_sorted.iter().map(|&v| v as u32).collect();

        Span::new(left_labels, right_labels, middle)
    }

    /// Builds the full `RewriteSpan` (L ← K → R) with explicit kernel hypergraph.
    ///
    /// This constructs the kernel as a hypergraph containing only the preserved
    /// vertices and the identity morphisms K → L and K → R.
    #[must_use]
    pub fn to_rewrite_span(&self) -> RewriteSpan {
        let preserved: BTreeSet<usize> = self.preserved_variables().into_iter().collect();

        // Build left hypergraph from pattern
        let mut left = Hypergraph::new();
        for edge in self.left() {
            left.add_hyperedge(edge.vertices().to_vec());
        }

        // Build right hypergraph from pattern
        let mut right = Hypergraph::new();
        for edge in self.right() {
            right.add_hyperedge(edge.vertices().to_vec());
        }

        // Kernel contains only preserved vertices (no edges — they transform)
        let mut kernel = Hypergraph::new();
        for &v in &preserved {
            kernel.add_vertex(Some(v));
        }

        // Identity morphisms: kernel vars map to themselves in L and R
        let left_map: HashMap<usize, usize> = preserved.iter().map(|&v| (v, v)).collect();
        let right_map: HashMap<usize, usize> = preserved.iter().map(|&v| (v, v)).collect();

        RewriteSpan::new(left, kernel, right, left_map, right_map)
    }
}

// ============================================================================
// RewriteSpan → Span
// ============================================================================

impl RewriteSpan {
    /// Converts this `RewriteSpan` to a catgraph `Span<u32>`.
    ///
    /// Uses the `left_map` and `right_map` morphisms to build the span's
    /// middle pairs, mapping kernel elements to their positions in L and R.
    /// Labels are vertex IDs (as `u32`).
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    #[must_use]
    pub fn to_span(&self) -> Span<u32> {
        let left_verts: Vec<usize> = self.left.vertices().collect();
        let right_verts: Vec<usize> = self.right.vertices().collect();

        // Build index maps
        let left_index: HashMap<usize, usize> = left_verts.iter()
            .enumerate()
            .map(|(i, &v)| (v, i))
            .collect();
        let right_index: HashMap<usize, usize> = right_verts.iter()
            .enumerate()
            .map(|(i, &v)| (v, i))
            .collect();

        // Each kernel vertex maps through left_map to L and right_map to R
        let mut middle: Vec<(usize, usize)> = Vec::new();
        for k_vert in self.kernel.vertices() {
            if let (Some(&l_vert), Some(&r_vert)) =
                (self.left_map.get(&k_vert), self.right_map.get(&k_vert))
            && let (Some(&l_idx), Some(&r_idx)) =
                (left_index.get(&l_vert), right_index.get(&r_vert))
            {
                middle.push((l_idx, r_idx));
            }
        }
        middle.sort_unstable();

        let left_labels: Vec<u32> = left_verts.iter().map(|&v| v as u32).collect();
        let right_labels: Vec<u32> = right_verts.iter().map(|&v| v as u32).collect();
        Span::new(left_labels, right_labels, middle)
    }
}

// ============================================================================
// HypergraphEvolution → Cospan chain
// ============================================================================

impl HypergraphEvolution {
    /// Converts the evolution into a chain of composable cospans.
    ///
    /// Each rewrite step Gi → Gi+1 produces a cospan:
    ///
    /// ```text
    ///     Gi_boundary ──→ apex ←── Gi+1_boundary
    /// ```
    ///
    /// where the apex is the union of all vertices from both states,
    /// and the boundary maps send each state's vertices to their
    /// positions in the apex. Preserved vertices map to the same
    /// apex element, creating the categorical "gluing."
    ///
    /// The returned cospans are composable: the right boundary of
    /// cospan i matches the left boundary of cospan i+1.
    ///
    /// # Returns
    ///
    /// A vector of cospans along the deterministic (root-to-last-node) path.
    /// Empty if the evolution has only the root node.
    #[must_use]
    pub fn to_cospan_chain(&self) -> Vec<Cospan<u32>> {
        let path = self.deterministic_path();
        if path.len() < 2 {
            return vec![];
        }

        path.windows(2)
            .map(|w| self.build_cospan_for_pair(w[0], w[1]))
            .collect()
    }

    /// Returns the deterministic path from root to the deepest node.
    ///
    /// Follows the first child at each step (deterministic choice).
    fn deterministic_path(&self) -> Vec<usize> {
        let mut path = vec![0]; // Start at root
        let mut current = 0;

        loop {
            // Find first child of current
            let mut found_child = false;
            for id in (current + 1)..self.node_count() {
                if let Some(node) = self.get_node(id)
                    && node.parent == Some(current)
                {
                    path.push(id);
                    current = id;
                    found_child = true;
                    break;
                }
            }
            if !found_child {
                break;
            }
        }

        path
    }

    /// Builds a cospan from a parent-child node pair.
    ///
    /// The apex is the union of both vertex sets, with labels as vertex IDs.
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn build_cospan_for_pair(&self, parent_id: usize, child_id: usize) -> Cospan<u32> {
        let parent = self.get_node(parent_id).unwrap();
        let child = self.get_node(child_id).unwrap();

        let parent_verts: Vec<usize> = parent.state.vertices().collect();
        let child_verts: Vec<usize> = child.state.vertices().collect();

        let mut apex_set: BTreeSet<usize> = BTreeSet::new();
        apex_set.extend(&parent_verts);
        apex_set.extend(&child_verts);
        let apex_sorted: Vec<usize> = apex_set.iter().copied().collect();

        let apex_index: HashMap<usize, usize> = apex_sorted.iter()
            .enumerate()
            .map(|(i, &v)| (v, i))
            .collect();

        let left: Vec<usize> = parent_verts.iter().map(|v| apex_index[v]).collect();
        let right: Vec<usize> = child_verts.iter().map(|v| apex_index[v]).collect();
        let middle: Vec<u32> = apex_sorted.iter().map(|&v| v as u32).collect();

        Cospan::new(left, right, middle)
    }

    /// Composes the deterministic cospan chain into a single composite cospan
    /// representing the global transformation from initial to final state.
    ///
    /// The composite's domain = root vertex IDs, codomain = final vertex IDs.
    ///
    /// # Panics
    ///
    /// Panics if adjacent cospans in the chain are not composable.
    ///
    /// # Errors
    ///
    /// Returns `CatgraphError::Composition` if the cospan chain is empty.
    pub fn compose_cospan_chain(&self) -> Result<Cospan<u32>, crate::errors::CatgraphError> {
        let chain = self.to_cospan_chain();
        chain.into_iter()
            .reduce(|acc, c| acc.compose(&c).expect("evolution cospans must be composable"))
            .ok_or_else(|| crate::errors::CatgraphError::Composition {
                message: "empty cospan chain".to_string()
            })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evolution_deterministic() {
        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];

        let evolution = HypergraphEvolution::run(&initial, &rules, 10);

        assert!(evolution.node_count() >= 2);
        assert_eq!(evolution.root().state.edge_count(), 1);
    }

    #[test]
    fn test_evolution_multiway() {
        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2], vec![1, 2, 3]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];

        let evolution = HypergraphEvolution::run_multiway(&initial, &rules, 3, 50);

        // Should have multiple branches
        assert!(evolution.node_count() > 1);
    }

    #[test]
    fn test_evolution_statistics() {
        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];

        let evolution = HypergraphEvolution::run(&initial, &rules, 5);
        let stats = evolution.statistics();

        assert!(stats.total_nodes >= 1);
        assert!(!stats.rule_applications.is_empty());
    }

    #[test]
    fn test_causal_invariance_trivial() {
        // Single path evolution is trivially invariant
        let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
        let rules = vec![RewriteRule::edge_split()];

        let evolution = HypergraphEvolution::run(&initial, &rules, 3);
        let result = evolution.analyze_causal_invariance();

        // No branches, so trivially invariant
        assert!(result.is_invariant || result.loops_analyzed == 0);
    }

    #[test]
    fn test_find_merges() {
        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2], vec![2, 3, 4]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];

        let evolution = HypergraphEvolution::run_multiway(&initial, &rules, 2, 20);
        let _merges = evolution.find_merges();

        // May or may not have merges depending on the specific evolution
    }

    #[test]
    fn test_path_to_root() {
        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];

        let evolution = HypergraphEvolution::run(&initial, &rules, 3);

        // Last node's path should include root
        let last_id = evolution.node_count() - 1;
        let path = evolution.path_to_root(last_id);

        assert!(path.contains(&0)); // Root is in path
        assert_eq!(path[0], last_id); // Starts with the node
        assert_eq!(*path.last().unwrap(), 0); // Ends at root
    }

    #[test]
    fn test_nodes_at_step() {
        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];

        let evolution = HypergraphEvolution::run(&initial, &rules, 3);

        let step_0 = evolution.nodes_at_step(0);
        assert_eq!(step_0.len(), 1);
        assert_eq!(step_0[0], 0);
    }

    // ── RewriteRule::to_span ───────────────────────────────────────────

    #[test]
    fn test_wolfram_a_to_bb_span() {
        // {0,1,2} → {0,1},{1,2}
        // L vars = {0,1,2}, R vars = {0,1,2}, K = {0,1,2} (all preserved)
        let rule = RewriteRule::wolfram_a_to_bb();
        let span = rule.to_span();

        assert_eq!(span.left(), &[0u32, 1, 2]);
        assert_eq!(span.right(), &[0u32, 1, 2]);
        assert_eq!(span.middle_pairs().len(), 3);

        // All three kernel elements map identity: (0,0), (1,1), (2,2)
        for &(l, r) in span.middle_pairs() {
            assert_eq!(l, r, "preserved vars should map to same index");
        }
    }

    #[test]
    fn test_edge_split_span() {
        // {0,1} → {0,2},{2,1}
        // L vars = {0,1}, R vars = {0,1,2}, K = {0,1}
        let rule = RewriteRule::edge_split();
        let span = rule.to_span();

        assert_eq!(span.left(), &[0u32, 1]);     // L has vars 0, 1
        assert_eq!(span.right(), &[0u32, 1, 2]); // R has vars 0, 1, 2
        assert_eq!(span.middle_pairs().len(), 2); // K = {0, 1}
    }

    #[test]
    fn test_triangle_rule_span() {
        // {0,1} → {0,1},{1,2},{2,0}
        // L vars = {0,1}, R vars = {0,1,2}, K = {0,1}
        let rule = RewriteRule::triangle();
        let span = rule.to_span();

        assert_eq!(span.left(), &[0u32, 1]);
        assert_eq!(span.right(), &[0u32, 1, 2]);
        assert_eq!(span.middle_pairs().len(), 2);
    }

    #[test]
    fn test_collapse_rule_span() {
        // {0,1},{1,2} → {0,2}
        // L vars = {0,1,2}, R vars = {0,2}, K = {0,2}
        let rule = RewriteRule::collapse();
        let span = rule.to_span();

        assert_eq!(span.left(), &[0u32, 1, 2]);
        assert_eq!(span.right(), &[0u32, 2]);
        assert_eq!(span.middle_pairs().len(), 2);
    }

    #[test]
    fn test_create_self_loop_span() {
        // {0,1} → {0,1},{1,1}
        // L vars = {0,1}, R vars = {0,1}, K = {0,1}
        let rule = RewriteRule::create_self_loop();
        let span = rule.to_span();

        assert_eq!(span.left(), &[0u32, 1]);
        assert_eq!(span.right(), &[0u32, 1]);
        assert_eq!(span.middle_pairs().len(), 2);
    }

    // ── RewriteRule::to_rewrite_span ───────────────────────────────────

    #[test]
    fn test_rewrite_span_roundtrip() {
        let rule = RewriteRule::wolfram_a_to_bb();
        let rspan = rule.to_rewrite_span();

        // Kernel should have 3 preserved vertices
        assert_eq!(rspan.kernel.vertex_count(), 3);
        // Left should have 1 edge (ternary)
        assert_eq!(rspan.left.edge_count(), 1);
        // Right should have 2 edges (binary)
        assert_eq!(rspan.right.edge_count(), 2);

        // Converting RewriteSpan to catgraph Span should match direct conversion
        let span_from_rule = rule.to_span();
        let span_from_rspan = rspan.to_span();

        assert_eq!(span_from_rule.left(), span_from_rspan.left());
        assert_eq!(span_from_rule.right(), span_from_rspan.right());
        assert_eq!(span_from_rule.middle_pairs(), span_from_rspan.middle_pairs());
    }

    #[test]
    fn test_edge_split_rewrite_span() {
        let rule = RewriteRule::edge_split();
        let rspan = rule.to_rewrite_span();

        // Kernel: vars {0,1} (preserved)
        assert_eq!(rspan.kernel.vertex_count(), 2);
        // Created var: 2 (only in right)
        assert!(rspan.right.vertices().any(|v| v == 2));
        assert!(!rspan.left.vertices().any(|v| v == 2));
    }

    // ── HypergraphEvolution::to_cospan_chain ───────────────────────────

    #[test]
    fn test_deterministic_evolution_cospan_chain() {
        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 3);

        let cospans = evolution.to_cospan_chain();

        // Should have at least 1 cospan (one rewrite step)
        assert!(!cospans.is_empty());

        // Each cospan's left boundary should have the parent's vertex count
        // and right boundary should have the child's vertex count
        for cospan in &cospans {
            assert!(!cospan.left_to_middle().is_empty());
            assert!(!cospan.right_to_middle().is_empty());
            // All indices should be valid (< middle.len())
            let middle_len = cospan.middle().len();
            assert!(cospan.left_to_middle().iter().all(|&i| i < middle_len));
            assert!(cospan.right_to_middle().iter().all(|&i| i < middle_len));
        }
    }

    #[test]
    fn test_cospan_chain_preserves_shared_vertices() {
        // A→BB: {0,1,2} → {0,1},{1,2}
        // All vertices preserved, so parent and child share the same apex positions
        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 1);

        let cospans = evolution.to_cospan_chain();
        assert_eq!(cospans.len(), 1);

        let cospan = &cospans[0];
        // For A→BB with no new vertices, parent and child have same vertices
        // so left and right should map to the same apex positions
        assert_eq!(cospan.left_to_middle(), cospan.right_to_middle());
    }

    #[test]
    fn test_cospan_chain_with_new_vertices() {
        // edge-split: {0,1} → {0,2},{2,1} — creates new vertex
        let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
        let rules = vec![RewriteRule::edge_split()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 1);

        let cospans = evolution.to_cospan_chain();
        assert_eq!(cospans.len(), 1);

        let cospan = &cospans[0];
        // Parent has 2 vertices {0,1}, child has 3 {0,1,2}
        assert_eq!(cospan.left_to_middle().len(), 2);
        assert_eq!(cospan.right_to_middle().len(), 3);
        // Apex should have 3 elements (union)
        assert_eq!(cospan.middle().len(), 3);
    }

    #[test]
    fn test_empty_evolution_no_cospans() {
        // No applicable rules → no steps → no cospans
        let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()]; // needs ternary edge
        let evolution = HypergraphEvolution::run(&initial, &rules, 10);

        let cospans = evolution.to_cospan_chain();
        assert!(cospans.is_empty());
    }

    #[test]
    fn test_multi_step_cospan_chain() {
        // Run multiple edge splits
        let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
        let rules = vec![RewriteRule::edge_split()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 3);

        let cospans = evolution.to_cospan_chain();
        assert_eq!(cospans.len(), 3);

        // Chain should be composable: right boundary of step i
        // has same size as left boundary of step i+1
        for i in 0..cospans.len() - 1 {
            assert_eq!(
                cospans[i].right_to_middle().len(),
                cospans[i + 1].left_to_middle().len(),
                "cospan chain boundary mismatch at step {}", i
            );
        }
    }

    // ── Cospan label values ─────────────────────────────────────────────

    #[test]
    fn test_cospan_apex_labels_are_vertex_ids() {
        // A→BB preserves all vertices: apex labels should be {0, 1, 2}
        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 1);

        let cospans = evolution.to_cospan_chain();
        assert_eq!(cospans.len(), 1);

        // Apex should contain actual vertex IDs
        assert_eq!(cospans[0].middle(), &[0u32, 1, 2]);
    }

    #[test]
    fn test_cospan_chain_composable_via_catgraph() {
        let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
        let rules = vec![RewriteRule::edge_split()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 3);

        let cospans = evolution.to_cospan_chain();
        assert_eq!(cospans.len(), 3);

        // Verify composability via catgraph's Composable trait
        for i in 0..cospans.len() - 1 {
            assert!(
                cospans[i].composable(&cospans[i + 1]).is_ok(),
                "cospans {} and {} should be composable", i, i + 1
            );
        }
    }

    // ── Span validity ──────────────────────────────────────────────────

    #[test]
    fn test_all_common_rules_produce_valid_spans() {
        let rules = vec![
            RewriteRule::wolfram_a_to_bb(),
            RewriteRule::edge_split(),
            RewriteRule::triangle(),
            RewriteRule::collapse(),
            RewriteRule::create_self_loop(),
        ];

        for rule in &rules {
            // to_span() calls Span::new() which calls assert_valid()
            let span = rule.to_span();
            assert!(!span.left().is_empty() || !span.right().is_empty(),
                "rule '{}' should produce non-trivial span", rule);

            // to_rewrite_span() + to_span() should also be valid
            let rspan = rule.to_rewrite_span();
            let _span2 = rspan.to_span();
        }
    }

    // ── Cospan composition ────────────────────────────────────────────

    #[test]
    fn test_compose_single_step() {
        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 1);

        let composite = evolution.compose_cospan_chain().unwrap();

        // For A→BB with all preserved, domain = codomain = {0, 1, 2}
        assert_eq!(composite.domain(), vec![0u32, 1, 2]);
        assert_eq!(composite.codomain(), vec![0u32, 1, 2]);
    }

    #[test]
    fn test_compose_multi_step() {
        let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
        let rules = vec![RewriteRule::edge_split()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 3);

        let composite = evolution.compose_cospan_chain().unwrap();

        // Domain should be root vertices
        assert_eq!(composite.domain(), vec![0u32, 1]);
    }

    #[test]
    fn test_compose_empty_chain_error() {
        let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()]; // won't match binary edge
        let evolution = HypergraphEvolution::run(&initial, &rules, 10);

        assert!(evolution.compose_cospan_chain().is_err());
    }
}
