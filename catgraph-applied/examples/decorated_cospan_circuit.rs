//! Worked example: F-decorated cospans with F(n) = edge set on n vertices.
//!
//! Minimal form of the textbook's Circ example (§6.4 Ex 6.79–6.86): each
//! decoration is a set of undirected edges between the apex vertices,
//! representing a "circuit" built from resistors.
//!
//! NOTE: `DecoratedCospan::compose` does not yet invoke `D::pushforward`
//! (pending upstream `Cospan::compose_with_quotient()`). This example uses
//! monoidal (parallel) composition only, which avoids the apex relabeling
//! issue. Series composition with pushforward will be demonstrated in a
//! v0.3.1 follow-up.

use catgraph::cospan::Cospan;
use catgraph::monoidal::Monoidal;
use catgraph_applied::decorated_cospan::{DecoratedCospan, Decoration};

#[derive(Clone, Debug, PartialEq)]
struct EdgeSet(Vec<(usize, usize)>);

struct Circuit;

impl Decoration for Circuit {
    type Apex = EdgeSet;
    fn empty(_: usize) -> EdgeSet {
        EdgeSet(vec![])
    }
    fn combine(mut a: EdgeSet, b: EdgeSet) -> EdgeSet {
        a.0.extend(b.0);
        a
    }
    fn pushforward(d: EdgeSet, quotient: &[usize]) -> EdgeSet {
        EdgeSet(
            d.0.into_iter()
                .map(|(u, v)| (quotient[u], quotient[v]))
                .collect(),
        )
    }
}

fn main() {
    // Two R1 resistor circuits, composed in parallel via monoidal product.
    let c1 = Cospan::<usize>::new(vec![0], vec![1], vec![0, 1]);
    let circ1 = DecoratedCospan::<usize, Circuit>::new(c1, EdgeSet(vec![(0, 1)]));

    let c2 = Cospan::<usize>::new(vec![0], vec![1], vec![0, 1]);
    let circ2 = DecoratedCospan::<usize, Circuit>::new(c2, EdgeSet(vec![(0, 1)]));

    let mut parallel = circ1;
    parallel.monoidal(circ2);
    println!(
        "parallel circuit has {} edges in decoration",
        parallel.decoration.0.len()
    );
}
