//! Integration tests for `DecoratedCospan::compose` invoking
//! `D::pushforward` through the pushout quotient (v0.3.1).

use catgraph::category::Composable;
use catgraph::cospan::Cospan;
use catgraph_applied::decorated_cospan::{DecoratedCospan, Decoration};

/// Edge-set decoration carrying its own apex size so the laxator can shift
/// c2's edge endpoints into the disjoint-union coordinate system.
///
/// The `Decoration` trait's `combine` signature does not receive the two
/// cardinalities, so implementations whose internal representation
/// references apex indices must carry that information inside their own
/// `Apex` type. Here we store `n`, the number of apex vertices the edge
/// indices are valid against.
#[derive(Clone, Debug, PartialEq)]
struct EdgeSet {
    n: usize,
    edges: Vec<(usize, usize)>,
}

struct Circuit;

impl Decoration for Circuit {
    type Apex = EdgeSet;
    fn empty(n: usize) -> EdgeSet {
        EdgeSet { n, edges: vec![] }
    }
    fn combine(a: EdgeSet, b: EdgeSet) -> EdgeSet {
        // Laxator φ: F(n_a) × F(n_b) → F(n_a + n_b). Shift b's endpoints
        // by n_a so they index into the disjoint union [0, n_a + n_b).
        let shift = a.n;
        let mut edges = a.edges;
        edges.extend(b.edges.into_iter().map(|(u, v)| (u + shift, v + shift)));
        EdgeSet {
            n: a.n + b.n,
            edges,
        }
    }
    fn pushforward(d: EdgeSet, quotient: &[usize]) -> EdgeSet {
        let new_n = quotient.iter().copied().max().map_or(0, |m| m + 1);
        EdgeSet {
            n: new_n,
            edges: d
                .edges
                .into_iter()
                .map(|(u, v)| (quotient[u], quotient[v]))
                .collect(),
        }
    }
}

#[test]
fn t2_1_circuit_edgeset_series_composition() {
    // Two 1-resistor cospans composed in series.
    // Each apex has two vertices sharing the same label (so the interface
    // label matches when composing). left leg points at vertex 0, right
    // leg at vertex 1, and there is one edge (0, 1).
    // After pushout, the right boundary of c1 and left boundary of c2
    // identify into a single shared vertex; the two edges must be
    // relabelled into the 3-vertex apex as [(0, 1), (1, 2)].
    let c1 = Cospan::<usize>::new(vec![0], vec![1], vec![0, 0]);
    let circ1 = DecoratedCospan::<usize, Circuit>::new(
        c1,
        EdgeSet {
            n: 2,
            edges: vec![(0, 1)],
        },
    );

    let c2 = Cospan::<usize>::new(vec![0], vec![1], vec![0, 0]);
    let circ2 = DecoratedCospan::<usize, Circuit>::new(
        c2,
        EdgeSet {
            n: 2,
            edges: vec![(0, 1)],
        },
    );

    let series = circ1.compose(&circ2).expect("series composition");

    assert_eq!(
        series.cospan.middle().len(),
        3,
        "3 apex vertices after pushout"
    );
    assert_eq!(series.decoration.edges.len(), 2, "2 edges after combining");
    for (u, v) in &series.decoration.edges {
        assert!(*u < 3 && *v < 3, "edge endpoint out of apex range");
    }
    let mut edges = series.decoration.edges.clone();
    edges.sort_unstable();
    assert_eq!(edges, vec![(0, 1), (1, 2)]);
}

/// Local trivial decoration — `catgraph_applied::decorated_cospan::Trivial`
/// is test-module-private, so we replicate the minimal unit decoration here.
struct LocalTrivial;

impl Decoration for LocalTrivial {
    type Apex = ();
    fn empty(_: usize) {}
    fn combine((): (), (): ()) {}
    fn pushforward((): (), _: &[usize]) {}
}

#[test]
fn t2_2_trivial_pushforward_is_unit() {
    // `pushforward` is not observably exercised here because `()` is the
    // only possible value for `LocalTrivial::Apex`. The non-vacuous
    // verification of pushforward wiring lives in T2.1 (the Circuit
    // EdgeSet test).
    let c1 = Cospan::<usize>::new(vec![0], vec![0], vec![0]);
    let d1 = DecoratedCospan::<usize, LocalTrivial>::new(c1, ());
    let c2 = Cospan::<usize>::new(vec![0], vec![0], vec![0]);
    let d2 = DecoratedCospan::<usize, LocalTrivial>::new(c2, ());
    let composed = d1.compose(&d2).unwrap();
    assert_eq!(composed.decoration, ());
}

#[test]
fn t2_3_petri_decoration_collapsed_quotient_preserves_transition_count() {
    // Quotient collapses both apex elements to 0, pushforward is a no-op
    // on transition count. Regression-guards the v0.3.0 behaviour that
    // composition preserves all transitions across the pushout.
    use catgraph_applied::petri_net::{PetriDecoration, Transition};
    use rust_decimal::Decimal;

    let c1 = Cospan::<char>::new(vec![0], vec![0], vec!['p']);
    let t1 = Transition::new(vec![(0, Decimal::ONE)], vec![]);
    let d1 = DecoratedCospan::<char, PetriDecoration<char>>::new(c1, vec![t1]);

    let c2 = Cospan::<char>::new(vec![0], vec![0], vec!['p']);
    let t2 = Transition::new(vec![], vec![(0, Decimal::ONE)]);
    let d2 = DecoratedCospan::<char, PetriDecoration<char>>::new(c2, vec![t2]);

    let composed = d1.compose(&d2).unwrap();
    // Both transitions preserved — quotient collapses both apex elements
    // to 0, so pushforward is a no-op on transition count.
    assert_eq!(composed.decoration.len(), 2);
}
