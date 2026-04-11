//! StepCount and Complexity trait API demonstration.
//!
//! Shows construction, Complexity trait methods (sequential, parallel),
//! operator overloads (+, +=), From conversions, ordering, and Display.

use catgraph::complexity::{Complexity, StepCount};

// ============================================================================
// Constructors
// ============================================================================

fn constructors() {
    println!("=== Constructors ===\n");

    let s = StepCount::new(7);
    println!("StepCount::new(7)  = {s}  (get = {})", s.get());

    let z = StepCount::zero();
    println!("StepCount::zero()  = {z}  (is_zero = {})", z.is_zero());

    let one = StepCount::one();
    println!("StepCount::one()   = {one}  (is_zero = {})", one.is_zero());

    // From<usize>
    let from: StepCount = 42.into();
    println!("StepCount::from(42) = {from}");

    // Into<usize>
    let back: usize = StepCount::new(99).into();
    println!("usize::from(99)    = {back}");
    println!();
}

// ============================================================================
// Complexity Trait Methods
// ============================================================================

fn complexity_trait() {
    println!("=== Complexity Trait ===\n");

    let a = StepCount::new(3);
    let b = StepCount::new(5);
    println!("a = {a}, b = {b}");

    // Sequential: cost of doing a then b (addition)
    let seq = a.sequential(&b);
    println!("a.sequential(&b)   = {seq}  (3 + 5 = {})", seq.as_steps());

    // Parallel: cost of doing a and b simultaneously (max)
    let par = a.parallel(&b);
    println!("a.parallel(&b)     = {par}  (max(3,5) = {})", par.as_steps());

    // Sequential with zero
    let zero = StepCount::zero();
    let seq_zero = a.sequential(&zero);
    println!("a.sequential(zero) = {seq_zero}  (identity element)");

    // Parallel with zero
    let par_zero = a.parallel(&zero);
    println!("a.parallel(zero)   = {par_zero}  (zero is neutral for max? no, max(3,0)=3)");
    println!();
}

// ============================================================================
// Operator Overloads
// ============================================================================

fn operators() {
    println!("=== Operator Overloads ===\n");

    let a = StepCount::new(10);
    let b = StepCount::new(25);

    // Add
    let sum = a + b;
    println!("{a} + {b} = {sum}");

    // AddAssign
    let mut acc = StepCount::new(100);
    acc += StepCount::new(50);
    println!("100 += 50 = {acc}");
    println!();
}

// ============================================================================
// Ordering
// ============================================================================

fn ordering() {
    println!("=== Ordering ===\n");

    let a = StepCount::new(3);
    let b = StepCount::new(7);
    let c = StepCount::new(3);

    println!("a={a}, b={b}, c={c}");
    println!("a < b  = {}", a < b);
    println!("a == c = {}", a == c);
    println!("b > c  = {}", b > c);

    // max/min using Ord
    let maximum = a.max(b);
    println!("max(a, b) = {maximum}");

    // Sort a vector
    let mut steps = [
        StepCount::new(5),
        StepCount::new(1),
        StepCount::new(9),
        StepCount::new(3),
    ];
    steps.sort();
    let sorted: Vec<String> = steps.iter().map(ToString::to_string).collect();
    println!("sorted = [{}]", sorted.join(", "));
    println!();
}

// ============================================================================
// Display Formatting
// ============================================================================

fn display() {
    println!("=== Display ===\n");

    println!("{}", StepCount::new(0));  // "0 steps"
    println!("{}", StepCount::new(1));  // "1 step"
    println!("{}", StepCount::new(42)); // "42 steps"
    println!();
}

fn main() {
    constructors();
    complexity_trait();
    operators();
    ordering();
    display();
}
