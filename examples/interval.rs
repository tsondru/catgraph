//! DiscreteInterval and ParallelIntervals API demonstration.
//!
//! Shows constructors, composition (both `compose` and `then`), containment,
//! intersection, tensor/direct_sum, structural equivalence vs exact equality,
//! and the TensorProduct trait.

use catgraph::bifunctor::TensorProduct;
use catgraph::interval::{DiscreteInterval, ParallelIntervals};

// ============================================================================
// Constructors
// ============================================================================

fn constructors() {
    println!("=== Constructors ===\n");

    let interval = DiscreteInterval::new(2, 7);
    println!("new(2, 7)          = {interval}  (cardinality {})", interval.cardinality());

    let ok = DiscreteInterval::try_new(3, 5);
    let err = DiscreteInterval::try_new(5, 3);
    println!("try_new(3, 5)      = {:?}", ok);
    println!("try_new(5, 3)      = {:?}", err);

    let single = DiscreteInterval::singleton(4);
    println!("singleton(4)       = {single}  (identity? {})", single.is_identity());

    let id = DiscreteInterval::identity(0);
    println!("identity(0)        = {id}  (steps {})", id.steps());
    println!();
}

// ============================================================================
// Composition
// ============================================================================

fn composition() {
    println!("=== Composition ===\n");

    // compose is right-to-left: f.compose(g) = f after g
    // requires g.end == f.start
    let g = DiscreteInterval::new(0, 3); // g: 0 -> 3
    let f = DiscreteInterval::new(3, 8); // f: 3 -> 8
    let fg = f.compose(g);
    println!("f={f}, g={g}");
    println!("f.compose(g)       = {:?}", fg);

    // Non-contiguous => None
    let h = DiscreteInterval::new(4, 9);
    println!("h={h}, g={g}");
    println!("h.compose(g)       = {:?}  (gap at 3..4)", h.compose(g));

    // then is left-to-right: a.then(b) requires a.end == b.start
    let a = DiscreteInterval::new(0, 3);
    let b = DiscreteInterval::new(3, 8);
    println!("\na={a}, b={b}");
    println!("a.then(b)          = {:?}", a.then(b));
    println!("a.is_composable_with(&b) = {}", a.is_composable_with(&b));

    // Identity composition
    let id = DiscreteInterval::identity(3);
    let result = f.compose(id);
    println!("\nf={f}, id={id}");
    println!("f.compose(id)      = {:?}  (identity law)", result);
    println!();
}

// ============================================================================
// Containment and Intersection
// ============================================================================

fn containment_and_intersection() {
    println!("=== Containment & Intersection ===\n");

    let outer = DiscreteInterval::new(1, 10);
    let inner = DiscreteInterval::new(3, 7);
    println!("outer={outer}, inner={inner}");
    println!("outer.contains(5)           = {}", outer.contains(5));
    println!("outer.contains(11)          = {}", outer.contains(11));
    println!("outer.contains_interval(&inner) = {}", outer.contains_interval(&inner));
    println!("inner.contains_interval(&outer) = {}", inner.contains_interval(&outer));

    let x = DiscreteInterval::new(1, 6);
    let y = DiscreteInterval::new(4, 9);
    let z = DiscreteInterval::new(7, 12);
    println!("\nx={x}, y={y}");
    println!("x.intersect(&y) = {:?}", x.intersect(&y));
    println!("x={x}, z={z}");
    println!("x.intersect(&z) = {:?}  (disjoint)", x.intersect(&z));
    println!();
}

// ============================================================================
// ParallelIntervals: tensor and direct_sum
// ============================================================================

fn parallel_intervals() {
    println!("=== ParallelIntervals ===\n");

    let mut pi = ParallelIntervals::new();
    println!("new()              branches={}, is_singleway={}", pi.branch_count(), pi.is_singleway());

    pi.add_branch(DiscreteInterval::new(0, 4));
    println!("after add_branch   branches={}, is_singleway={}", pi.branch_count(), pi.is_singleway());

    let branch_b = ParallelIntervals::from_branch(DiscreteInterval::new(10, 18));
    let combined = pi.tensor(branch_b);
    println!(
        "tensor             branches={}, total_complexity={}, max_complexity={}",
        combined.branch_count(),
        combined.total_complexity(),
        combined.max_complexity(),
    );

    let left = ParallelIntervals::from_branch(DiscreteInterval::new(0, 2));
    let right = ParallelIntervals::from_branch(DiscreteInterval::new(5, 8));
    let sum = left.direct_sum(right);
    println!(
        "direct_sum         branches={}, total_complexity={}",
        sum.branch_count(),
        sum.total_complexity(),
    );
    println!();
}

// ============================================================================
// Structural Equivalence vs Exact Equality
// ============================================================================

fn equivalence() {
    println!("=== Structural Equivalence vs Exact Equality ===\n");

    // Same cardinalities, different positions
    let a = {
        let mut p = ParallelIntervals::new();
        p.add_branch(DiscreteInterval::new(0, 2)); // card 3
        p.add_branch(DiscreteInterval::new(5, 9)); // card 5
        p
    };
    let b = {
        let mut p = ParallelIntervals::new();
        p.add_branch(DiscreteInterval::new(10, 14)); // card 5
        p.add_branch(DiscreteInterval::new(20, 22)); // card 3
        p
    };
    println!("a = [0,2]+[5,9],  b = [10,14]+[20,22]");
    println!("structurally_equivalent = {}", a.structurally_equivalent(&b));
    println!("exactly_equal           = {}", a.exactly_equal(&b));

    let c = a.clone();
    println!("\na == c (clone)");
    println!("structurally_equivalent = {}", a.structurally_equivalent(&c));
    println!("exactly_equal           = {}", a.exactly_equal(&c));
    println!();
}

// ============================================================================
// TensorProduct trait
// ============================================================================

fn tensor_product_trait() {
    println!("=== TensorProduct Trait ===\n");

    let unit = ParallelIntervals::unit();
    println!("unit().is_unit()   = {}", unit.is_unit());

    let a = ParallelIntervals::from_branch(DiscreteInterval::new(0, 3));
    println!("a.is_unit()        = {}", a.is_unit());

    // Unit laws: unit ⊗ a == a, a ⊗ unit == a
    let left_id = ParallelIntervals::unit().tensor(a.clone());
    let right_id = a.clone().tensor(ParallelIntervals::unit());
    println!("unit.tensor(a) == a  (left unit)  = {}", left_id.exactly_equal(&a));
    println!("a.tensor(unit) == a  (right unit) = {}", right_id.exactly_equal(&a));

    // Associativity: (a⊗b)⊗c == a⊗(b⊗c)
    let b = ParallelIntervals::from_branch(DiscreteInterval::new(5, 8));
    let c = ParallelIntervals::from_branch(DiscreteInterval::new(10, 12));
    let left_assoc = a.clone().tensor(b.clone()).tensor(c.clone());
    let right_assoc = a.tensor(b.tensor(c));
    println!(
        "(a⊗b)⊗c == a⊗(b⊗c) = {}",
        left_assoc.exactly_equal(&right_assoc),
    );
    println!();
}

fn main() {
    constructors();
    composition();
    containment_and_intersection();
    parallel_intervals();
    equivalence();
    tensor_product_trait();
}
