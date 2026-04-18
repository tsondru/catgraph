//! Integration tests for `Cospan::compose_with_quotient` — the additive
//! pushout-quotient API added in v0.11.3.

use catgraph::category::{Composable, HasIdentity};
use catgraph::cospan::Cospan;

mod common;
use common::cospan_eq;

#[test]
fn t1_1_identity_compose_quotient_concatenates_ranges() {
    // id(3) ∘ id(3): left cospan has middle [a,b,c] with both legs = [0,1,2];
    // right cospan is the same. Pushout merges right leg of first with left
    // leg of second pointwise, so the quotient maps both middles surjectively
    // onto the shared pushout [0,1,2].
    let left = Cospan::<char>::identity(&vec!['a', 'b', 'c']);
    let right = Cospan::<char>::identity(&vec!['a', 'b', 'c']);

    let (composed, quotient) = left
        .compose_with_quotient(&right)
        .expect("identities compose");

    // Quotient length = self.middle.len() + other.middle.len()
    assert_eq!(quotient.len(), 6);
    // First 3 entries map self.middle indices [0,1,2] into the pushout
    assert_eq!(&quotient[..3], &[0, 1, 2]);
    // Next 3 entries map other.middle indices [0,1,2] to the *same* classes
    assert_eq!(&quotient[3..], &[0, 1, 2]);
    // Sanity: composed cospan has 3 middle elements
    assert_eq!(composed.middle().len(), 3);
}

#[test]
fn t1_2_surjective_coequalizer_merges_shared_element() {
    // Left cospan: domain [a], codomain [b], middle [a, b], legs [0], [1]
    // Right cospan: domain [b], codomain [c], middle [b, c], legs [0], [1]
    // Compose glues Left.right (= middle[1] = b) to Right.left (= middle[0] = b)
    // Pushout apex has 3 classes: {a}, {b (shared)}, {c}
    let left = Cospan::<char>::new(vec![0], vec![1], vec!['a', 'b']);
    let right = Cospan::<char>::new(vec![0], vec![1], vec!['b', 'c']);

    let (composed, quotient) = left.compose_with_quotient(&right).unwrap();

    assert_eq!(composed.middle().len(), 3);
    assert_eq!(quotient.len(), 4); // 2 + 2

    // quotient[0] = class of left.middle[0] = 'a'
    // quotient[1] = class of left.middle[1] = 'b' (shared with right.middle[0])
    // quotient[2] = class of right.middle[0] = 'b' (shared — same as quotient[1])
    // quotient[3] = class of right.middle[1] = 'c'
    assert_eq!(&quotient[..], &[0, 1, 1, 2], "deterministic pushout quotient");
    assert_eq!(quotient[1], quotient[2], "shared 'b' collapses to one class");
    assert_ne!(quotient[0], quotient[1], "'a' stays separate from 'b'");
    assert_ne!(quotient[3], quotient[1], "'c' stays separate from 'b'");
}

#[test]
fn t1_3_roundtrip_with_plain_compose() {
    // compose_with_quotient(a, b).0 must equal compose(a, b) for several inputs.
    let cases = [
        (
            Cospan::<char>::identity(&vec!['x']),
            Cospan::<char>::identity(&vec!['x']),
        ),
        (
            Cospan::<char>::new(vec![0], vec![1], vec!['p', 'q']),
            Cospan::<char>::new(vec![0], vec![1], vec!['q', 'r']),
        ),
    ];
    for (a, b) in &cases {
        let via_compose = a.compose(b).unwrap();
        let (via_quotient, _) = a.compose_with_quotient(b).unwrap();
        assert!(cospan_eq(&via_compose, &via_quotient));
    }
}
