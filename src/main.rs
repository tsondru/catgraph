use either::Either::{Left, Right};
use petgraph::dot::Dot;

use catgraph::named_cospan::NamedCospan;

fn main() {
    let mut x = NamedCospan::<u32, &'static str, &'static str>::empty();
    x.add_boundary_node_unknown_target(0, Right("out1"));
    x.add_boundary_node_known_target(0, Right("out2"));
    x.add_boundary_node_known_target(0, Left("in1"));
    x.add_boundary_node_unknown_target(0, Right("out4"));
    x.add_boundary_node_unknown_target(1, Left("in2"));
    x.add_boundary_node_unknown_target(1, Left("in3"));
    x.connect_pair(Left("in2"), Left("in3"));

    let (_, _, _, graph) = x.to_graph(
        |lambda| (lambda.to_string(), ()),
        |port_type_color, port_name| {
            *port_type_color = format!("{} of type {}", port_name, *port_type_color);
        },
    );

    println!("{:?}", Dot::new(&graph));

    x.connect_pair(Right("out1"), Right("out4"));

    let (_, _, _, graph) = x.to_graph(
        |lambda| (lambda.to_string(), ()),
        |port_type_color, port_name| {
            *port_type_color = format!("{} of type {}", port_name, *port_type_color);
        },
    );

    println!("{:?}", Dot::new(&graph));
}
