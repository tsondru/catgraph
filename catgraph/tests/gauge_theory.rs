//! Integration tests for gauge theory module.
//!
//! Tests structure constants, plaquette/total action functions,
//! and `HypergraphLattice` construction, state management, DPO
//! rewriting with holonomy, Wilson loops, and causal invariance.

#![allow(clippy::float_cmp)]

use catgraph::hypergraph::{
    plaquette_action, total_action, GaugeGroup, Hypergraph, HypergraphLattice,
    HypergraphRewriteGroup, RewriteRule,
};

// ---------------------------------------------------------------------------
// Structure constants
// ---------------------------------------------------------------------------

#[test]
fn structure_constant_antisymmetric() {
    let group = HypergraphRewriteGroup::new(4);

    // f^{abc} = -f^{bac} when c coincides with a or b.
    // The simplified model uses sign(b > a) for c == a and sign(a > b)
    // for c == b, giving antisymmetry in those branches.
    for (a, b, c) in [(0, 1, 0), (0, 1, 1), (1, 2, 1), (1, 2, 2), (0, 3, 0), (0, 3, 3)] {
        let forward = group.structure_constant_for(a, b, c);
        let swapped = group.structure_constant_for(b, a, c);
        assert!(
            (forward + swapped).abs() < 1e-12,
            "f^{{{a},{b},{c}}} = {forward}, f^{{{b},{a},{c}}} = {swapped}; sum should be 0"
        );
    }

    // When all three indices are distinct the simplified model returns 1.0
    // for both orderings (non-antisymmetric -- acknowledged simplification).
    assert_eq!(group.structure_constant_for(0, 1, 2), 1.0);
    assert_eq!(group.structure_constant_for(1, 0, 2), 1.0);
}

#[test]
fn structure_constant_zero_when_equal() {
    let group = HypergraphRewriteGroup::new(4);

    // f^{aac} = 0 for all a, c
    for a in 0..4 {
        for c in 0..4 {
            assert_eq!(
                group.structure_constant_for(a, a, c),
                0.0,
                "f^{{{a},{a},{c}}} should be 0"
            );
        }
    }
}

#[test]
fn structure_constant_out_of_range() {
    let group = HypergraphRewriteGroup::new(3);

    // Any index >= num_rules yields 0
    assert_eq!(group.structure_constant_for(3, 0, 1), 0.0);
    assert_eq!(group.structure_constant_for(0, 3, 1), 0.0);
    assert_eq!(group.structure_constant_for(0, 1, 3), 0.0);
    assert_eq!(group.structure_constant_for(5, 5, 5), 0.0);
}

#[test]
fn trait_constants_correct() {
    assert_eq!(HypergraphRewriteGroup::LIE_ALGEBRA_DIM, 3);
    let is_abelian = HypergraphRewriteGroup::IS_ABELIAN;
    assert!(!is_abelian);
    assert_eq!(HypergraphRewriteGroup::SPACETIME_DIM, 1);
    assert_eq!(HypergraphRewriteGroup::name(), "HypergraphRewrite");
}

// ---------------------------------------------------------------------------
// Plaquette and total action
// ---------------------------------------------------------------------------

#[test]
fn plaquette_action_flat() {
    assert!((plaquette_action(1.0)).abs() < 1e-12);
}

#[test]
fn plaquette_action_curved() {
    let action = plaquette_action(0.5);
    assert!(action > 0.0);
    // -ln(0.5) = ln(2) ≈ 0.6931
    assert!((action - 2.0_f64.ln()).abs() < 1e-12);
}

#[test]
fn plaquette_action_zero_holonomy() {
    assert!(plaquette_action(0.0).is_infinite());
}

#[test]
fn total_action_sums() {
    let expected = 2.0 * plaquette_action(0.5);
    let actual = total_action(&[1.0, 0.5, 0.5]);
    assert!(
        (actual - expected).abs() < 1e-12,
        "total_action([1.0, 0.5, 0.5]) = {actual}, expected {expected}"
    );
}

// ---------------------------------------------------------------------------
// HypergraphLattice construction
// ---------------------------------------------------------------------------

#[test]
fn lattice_1d_construction() {
    let lattice: HypergraphLattice<1> =
        HypergraphLattice::new([5], HypergraphRewriteGroup::new(3), vec![]);

    assert_eq!(lattice.dimensions(), &[5]);
    assert_eq!(lattice.group().num_rules(), 3);
    assert_eq!(lattice.step_count(), 0);
    assert_eq!(lattice.site_count(), 0); // no states populated yet
}

#[test]
fn lattice_2d_construction() {
    let lattice: HypergraphLattice<2> =
        HypergraphLattice::new([4, 4], HypergraphRewriteGroup::new(2), vec![]);

    assert_eq!(lattice.dimensions(), &[4, 4]);
    assert_eq!(lattice.group().num_rules(), 2);
    assert_eq!(lattice.site_count(), 0);
}

// ---------------------------------------------------------------------------
// State management
// ---------------------------------------------------------------------------

#[test]
fn set_and_get_state() {
    let mut lattice: HypergraphLattice<2> =
        HypergraphLattice::new([4, 4], HypergraphRewriteGroup::new(3), vec![]);

    let graph = Hypergraph::from_edges(vec![vec![0, 1, 2], vec![2, 3]]);
    lattice.set_state(&[1, 2], graph);

    let retrieved = lattice.get_state(&[1, 2]);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().vertex_count(), 4);

    let retrieved2 = lattice.get_state(&[1, 2]);
    assert_eq!(retrieved2.unwrap().edge_count(), 2);

    // Unoccupied site returns None
    assert!(lattice.get_state(&[0, 0]).is_none());
}

// ---------------------------------------------------------------------------
// apply_rewrite with DPO rewriting
// ---------------------------------------------------------------------------

#[test]
fn apply_rewrite_dpo_splits_ternary_edge() {
    let rule = RewriteRule::wolfram_a_to_bb();
    let mut lattice: HypergraphLattice<1> =
        HypergraphLattice::new([5], HypergraphRewriteGroup::new(3), vec![rule]);

    // Place a ternary edge at site [2]
    let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
    lattice.set_state(&[2], initial);

    assert!(lattice.apply_rewrite(&[2], 0));
    assert_eq!(lattice.step_count(), 1);

    // The ternary edge should have been replaced by two binary edges
    let state = lattice.get_state(&[2]).unwrap();
    assert_eq!(state.edge_count(), 2);
}

// ---------------------------------------------------------------------------
// Holonomy values after apply_rewrite
// ---------------------------------------------------------------------------

#[test]
fn holonomy_after_rewrite_is_edge_ratio() {
    // wolfram_a_to_bb: 1 ternary edge → 2 binary edges
    // holonomy = edges_after / edges_before = 2.0 / 1.0 = 2.0
    let rule = RewriteRule::wolfram_a_to_bb();
    let mut lattice: HypergraphLattice<1> =
        HypergraphLattice::new([5], HypergraphRewriteGroup::new(3), vec![rule]);

    let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
    lattice.set_state(&[2], initial);

    assert!(lattice.apply_rewrite(&[2], 0));
    assert_eq!(lattice.step_count(), 1);

    // apply_rewrite stores transition with self-loop key (site, site).
    // A single-site Wilson loop path picks up exactly that key once.
    let s = [2];
    let holonomy = lattice.wilson_loop(&[&s]);
    assert!(
        (holonomy - 2.0).abs() < 1e-12,
        "holonomy should be 2.0 (2 edges / 1 edge), got {holonomy}"
    );
}

#[test]
fn holonomy_unchanged_after_failed_rewrite() {
    let rule = RewriteRule::wolfram_a_to_bb();
    let mut lattice: HypergraphLattice<1> =
        HypergraphLattice::new([5], HypergraphRewriteGroup::new(3), vec![rule]);

    let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
    lattice.set_state(&[2], initial);

    // First rewrite succeeds: 1 ternary → 2 binary
    assert!(lattice.apply_rewrite(&[2], 0));
    assert_eq!(lattice.step_count(), 1);

    let s = [2];
    let holonomy_after_first = lattice.wilson_loop(&[&s]);
    assert!(
        (holonomy_after_first - 2.0).abs() < 1e-12,
        "first rewrite holonomy should be 2.0, got {holonomy_after_first}"
    );

    // Second rewrite on same site fails — no ternary edge left to match
    assert!(!lattice.apply_rewrite(&[2], 0));
    assert_eq!(lattice.step_count(), 1, "step_count must not increment on failure");

    // Holonomy unchanged: the failed rewrite does not touch transitions
    let holonomy_after_second = lattice.wilson_loop(&[&s]);
    assert!(
        (holonomy_after_second - 2.0).abs() < 1e-12,
        "holonomy should remain 2.0 after failed rewrite, got {holonomy_after_second}"
    );
}

#[test]
fn plaquette_action_documents_flat_for_large_holonomy() {
    // After a successful rewrite, holonomy = 2.0 (> 1.0).
    // plaquette_action clamps h >= 1.0 to action = 0.0 (flat).
    // This documents current behavior: the plaquette_action function treats
    // any holonomy >= 1.0 as "flat", even though h = 2.0 represents a
    // non-trivial topology change. Serves as a regression test.
    let rule = RewriteRule::wolfram_a_to_bb();
    let mut lattice: HypergraphLattice<1> =
        HypergraphLattice::new([5], HypergraphRewriteGroup::new(3), vec![rule]);

    let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
    lattice.set_state(&[2], initial);

    assert!(lattice.apply_rewrite(&[2], 0));

    let s = [2];
    let action = lattice.plaquette_action(&[&s]);
    assert!(
        action.abs() < 1e-12,
        "plaquette_action should be 0.0 for holonomy >= 1.0, got {action}"
    );
}

#[test]
fn wilson_loop_2d_self_loop_transitions() {
    // Documents that apply_rewrite stores self-loop keys (site, site),
    // so inter-site edges in a Wilson loop path get default holonomy 1.0.
    // Only the self-loop segments contribute non-trivial holonomy.
    let rule = RewriteRule::wolfram_a_to_bb();
    let mut lattice: HypergraphLattice<2> =
        HypergraphLattice::new([4, 4], HypergraphRewriteGroup::new(3), vec![rule]);

    // Place ternary edges at two adjacent sites
    let s0 = [1, 1];
    let s1 = [2, 1];
    lattice.set_state(&s0, Hypergraph::from_edges(vec![vec![0, 1, 2]]));
    lattice.set_state(&s1, Hypergraph::from_edges(vec![vec![0, 1, 2]]));

    // Rewrite both sites: each gets holonomy 2.0 on its self-loop key
    assert!(lattice.apply_rewrite(&s0, 0));
    assert!(lattice.apply_rewrite(&s1, 0));
    assert_eq!(lattice.step_count(), 2);

    // Wilson loop through a 4-site plaquette including both rewritten sites.
    // Path: s0 → s1 → [2,2] → [1,2] → back to s0
    // Transitions looked up:
    //   (s0, s1)     — inter-site, not stored → default 1.0
    //   (s1, [2,2])  — inter-site, not stored → default 1.0
    //   ([2,2],[1,2])— inter-site, not stored → default 1.0
    //   ([1,2], s0)  — inter-site, not stored → default 1.0
    // Product = 1.0 (all inter-site, no self-loop keys on path)
    let s2 = [2, 2];
    let s3 = [1, 2];
    let path: Vec<&[usize; 2]> = vec![&s0, &s1, &s2, &s3];
    let holonomy = lattice.wilson_loop(&path);
    assert!(
        (holonomy - 1.0).abs() < 1e-12,
        "inter-site Wilson loop should be 1.0 (self-loop keys only), got {holonomy}"
    );

    // Contrast: single-site loops DO pick up the rewrite holonomy
    let h0 = lattice.wilson_loop(&[&s0]);
    let h1 = lattice.wilson_loop(&[&s1]);
    assert!(
        (h0 - 2.0).abs() < 1e-12,
        "self-loop at s0 should be 2.0, got {h0}"
    );
    assert!(
        (h1 - 2.0).abs() < 1e-12,
        "self-loop at s1 should be 2.0, got {h1}"
    );
}

#[test]
fn apply_rewrite_no_match_returns_false() {
    let rule = RewriteRule::wolfram_a_to_bb(); // expects ternary edge
    let mut lattice: HypergraphLattice<1> =
        HypergraphLattice::new([5], HypergraphRewriteGroup::new(3), vec![rule]);

    // Place a binary edge -- won't match the A→BB rule
    let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
    lattice.set_state(&[2], initial);

    assert!(!lattice.apply_rewrite(&[2], 0));
    assert_eq!(lattice.step_count(), 0);
}

#[test]
fn apply_rewrite_invalid_site() {
    let mut lattice: HypergraphLattice<1> =
        HypergraphLattice::new([5], HypergraphRewriteGroup::new(3), vec![]);

    // Site [5] is out of bounds for dimension size 5 (valid: 0..4)
    assert!(!lattice.apply_rewrite(&[5], 0));
    assert_eq!(lattice.step_count(), 0);
}

#[test]
fn apply_rewrite_invalid_rule() {
    let mut lattice: HypergraphLattice<1> =
        HypergraphLattice::new([5], HypergraphRewriteGroup::new(2), vec![]);

    // No rules at all -- any rule index should fail
    assert!(!lattice.apply_rewrite(&[1], 0));
    assert_eq!(lattice.step_count(), 0);
}

// ---------------------------------------------------------------------------
// Wilson loops and causal invariance
// ---------------------------------------------------------------------------

#[test]
fn wilson_loop_empty_path() {
    let lattice: HypergraphLattice<1> =
        HypergraphLattice::new([5], HypergraphRewriteGroup::new(3), vec![]);

    let path: Vec<&[usize; 1]> = vec![];
    assert_eq!(lattice.wilson_loop(&path), 1.0);
}

#[test]
fn wilson_loop_no_transitions() {
    let lattice: HypergraphLattice<2> =
        HypergraphLattice::new([4, 4], HypergraphRewriteGroup::new(3), vec![]);

    // Path over sites with no recorded transitions -> holonomy = 1.0
    let s0 = [0, 0];
    let s1 = [1, 0];
    let s2 = [1, 1];
    let s3 = [0, 1];
    let path: Vec<&[usize; 2]> = vec![&s0, &s1, &s2, &s3];

    assert_eq!(lattice.wilson_loop(&path), 1.0);
}

#[test]
fn is_causally_invariant_trivial() {
    let lattice: HypergraphLattice<1> =
        HypergraphLattice::new([5], HypergraphRewriteGroup::new(3), vec![]);

    // Trivial path (single site loop) with no transitions -> invariant
    let s0 = [2];
    let path: Vec<&[usize; 1]> = vec![&s0];
    assert!(lattice.is_causally_invariant(&path));
}

#[test]
fn find_wilson_loops_2d() {
    let mut lattice: HypergraphLattice<2> =
        HypergraphLattice::new([3, 3], HypergraphRewriteGroup::new(2), vec![]);

    lattice.find_wilson_loops(4);

    // A 3x3 grid has (3-1)*(3-1) = 4 elementary plaquettes
    let loops = lattice.recorded_loops();
    assert_eq!(loops.len(), 4, "3x3 lattice should have 4 plaquettes");

    // All holonomies should be 1.0 (no transitions recorded)
    for (sites, holonomy) in loops {
        assert_eq!(*holonomy, 1.0);
        assert_eq!(sites.len(), 4, "each plaquette has 4 corners");
    }
}
