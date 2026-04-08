//! HypergraphFunctor API demonstration.
//!
//! Shows how RelabelingFunctor maps cospans between different label types
//! while preserving all categorical structure (Fong-Spivak §2.3, Eq. 12).

use catgraph::{
    category::{Composable, ComposableMutating, HasIdentity},
    cospan::Cospan,
    frobenius::FrobeniusMorphism,
    hypergraph_category::HypergraphCategory,
    hypergraph_functor::{CospanToFrobeniusFunctor, HypergraphFunctor, RelabelingFunctor},
    monoidal::Monoidal,
};

fn main() {
    println!("=== HypergraphFunctor: Relabeling ===\n");

    // Create a functor that relabels char → u32
    let functor = RelabelingFunctor::new(|c: char| c as u32);

    // --- Object mapping ---
    println!("Object mapping:");
    println!("  'a' → {}", functor.map_ob('a'));
    println!("  'z' → {}", functor.map_ob('z'));

    // --- Morphism mapping: identity ---
    println!("\nIdentity preservation:");
    let id = Cospan::<char>::identity(&vec!['a', 'b']);
    let mapped_id = functor.map_mor(&id).unwrap();
    println!("  id(['a','b'])   domain = {:?}", id.domain());
    println!("  F(id)           domain = {:?}", mapped_id.domain());

    // --- Frobenius generators ---
    println!("\nFrobenius generator preservation:");

    let eta = Cospan::<char>::unit('x');
    let mapped_eta = functor.map_mor(&eta).unwrap();
    println!("  η('x'):    [] → {:?}", eta.codomain());
    println!(
        "  F(η('x')): [] → {:?}  (= η({}))",
        mapped_eta.codomain(),
        functor.map_ob('x')
    );

    let mu = Cospan::<char>::multiplication('x');
    let mapped_mu = functor.map_mor(&mu).unwrap();
    println!("  μ('x'):    {:?} → {:?}", mu.domain(), mu.codomain());
    println!(
        "  F(μ('x')): {:?} → {:?}",
        mapped_mu.domain(),
        mapped_mu.codomain()
    );

    // --- Functoriality: composition ---
    println!("\nFunctoriality (composition):");
    let delta = Cospan::<char>::comultiplication('a');
    let mu_a = Cospan::<char>::multiplication('a');
    let composed = delta.compose(&mu_a).unwrap();
    let mapped_composed = functor.map_mor(&composed).unwrap();
    let mapped_delta = functor.map_mor(&delta).unwrap();
    let mapped_mu_a = functor.map_mor(&mu_a).unwrap();
    let compose_mapped = mapped_delta.compose(&mapped_mu_a).unwrap();
    println!("  F(δ;μ) middle     = {:?}", mapped_composed.middle());
    println!("  F(δ);F(μ) middle  = {:?}", compose_mapped.middle());
    println!(
        "  Equal? {}",
        mapped_composed.middle() == compose_mapped.middle()
            && mapped_composed.left_to_middle() == compose_mapped.left_to_middle()
            && mapped_composed.right_to_middle() == compose_mapped.right_to_middle()
    );

    // --- Monoidal preservation ---
    println!("\nMonoidal preservation:");
    let g = Cospan::<char>::unit('a');
    let h = Cospan::<char>::counit('b');
    let mut tensor = g.clone();
    tensor.monoidal(h.clone());
    let mapped_tensor = functor.map_mor(&tensor).unwrap();
    let mut mapped_parts = functor.map_mor(&g).unwrap();
    mapped_parts.monoidal(functor.map_mor(&h).unwrap());
    println!("  F(g⊗h) domain   = {:?}", mapped_tensor.domain());
    println!("  F(g)⊗F(h) domain = {:?}", mapped_parts.domain());

    println!("\n=== CospanToFrobeniusFunctor: Cospan → FrobeniusMorphism ===\n");

    let ctf = CospanToFrobeniusFunctor::<String>::new();

    // Object mapping is identity
    println!("Object mapping (identity):");
    println!("  'a' → '{}'", ctf.map_ob('a'));

    // Morphism mapping: merge and split
    println!("\nMorphism mapping (merge/split):");
    let merge = Cospan::new(vec![0, 0], vec![0], vec!['a']);
    let mapped_merge: FrobeniusMorphism<char, String> = ctf.map_mor(&merge).unwrap();
    println!(
        "  merge: {:?} → {:?}  (cospan)",
        merge.domain(),
        merge.codomain()
    );
    println!(
        "  F(merge): {:?} → {:?}  (frobenius morphism)",
        mapped_merge.domain(),
        mapped_merge.codomain()
    );

    let split = Cospan::new(vec![0], vec![0, 0], vec!['a']);
    let mapped_split: FrobeniusMorphism<char, String> = ctf.map_mor(&split).unwrap();
    println!(
        "  split: {:?} → {:?}  (cospan)",
        split.domain(),
        split.codomain()
    );
    println!(
        "  F(split): {:?} → {:?}  (frobenius morphism)",
        mapped_split.domain(),
        mapped_split.codomain()
    );

    // Functoriality: compose split then merge
    println!("\nFunctoriality:");
    let composed = split.compose(&merge).unwrap();
    let mapped_composed: FrobeniusMorphism<char, String> = ctf.map_mor(&composed).unwrap();
    println!(
        "  F(split;merge): {:?} → {:?}",
        mapped_composed.domain(),
        mapped_composed.codomain()
    );
}
