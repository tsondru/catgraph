//! Lawvere metric spaces — categories enriched over [`Tropical`] (= `[0, ∞]`
//! with min-plus semiring structure). Pedagogical references: CTFP §28.5,
//! Lawvere 1973 *Metric spaces, generalized logic, and closed categories*.
//!
//! A Lawvere metric space `(T, d)` is a set T with a distance function
//! `d: T × T → [0, ∞]` satisfying:
//! - `d(x, x) = 0` (identity / reflexivity)
//! - `d(x, z) ≤ d(x, y) + d(y, z)` (triangle inequality)
//!
//! Unlike classical metric spaces, Lawvere metrics are not required to be
//! symmetric (`d(x, y) = d(y, x)` not assumed) or have `d(x, y) = 0 → x = y`
//! (non-separation allowed). This generalisation is what lets BTV 2021
//! (arXiv:2106.07890) use Lawvere metrics as the distance-valued hom of
//! language categories, and BV 2025 (arXiv:2501.06662) compute magnitude
//! over such enrichments.

use std::collections::HashMap;
use std::hash::Hash;

use num::Zero;

use crate::{
    enriched::EnrichedCategory,
    rig::{BaseChange, Tropical, UnitInterval},
};

/// A Lawvere metric space enriched over [`Tropical`].
///
/// Objects live in a `Vec<T>` (insertion-ordered); distances are stored in a
/// [`HashMap`] keyed by `(T, T)`. Unset distances default to
/// `Tropical::zero() = Tropical(+∞)` — "unreachable" under shortest-path
/// semantics.
#[derive(Debug, Clone)]
pub struct LawvereMetricSpace<T: Clone + Eq + Hash> {
    objects: Vec<T>,
    distances: HashMap<(T, T), Tropical>,
}

impl<T: Clone + Eq + Hash> LawvereMetricSpace<T> {
    /// Construct an empty metric space over a fixed object list. All
    /// distances start at `Tropical(+∞)`; use
    /// [`set_distance`](Self::set_distance) to populate.
    #[must_use]
    pub fn new(objects: Vec<T>) -> Self {
        Self {
            objects,
            distances: HashMap::new(),
        }
    }

    /// Set the directed distance from `a` to `b` (overwriting any prior
    /// value). Lawvere metrics are not required to be symmetric — setting
    /// `d(a, b)` does not set `d(b, a)`.
    pub fn set_distance(&mut self, a: T, b: T, d: Tropical) {
        self.distances.insert((a, b), d);
    }

    /// Distance from `a` to `b`. Returns `Tropical(+∞)` if unset.
    ///
    /// Convention: unset distance = `Tropical::zero()` = `Tropical(+∞)` in
    /// the min-plus semiring (see `rig.rs:161-164`). Semantically: "no edge" /
    /// "unreachable". Under min-plus multiplication (= real addition),
    /// `+∞ + anything = +∞`, so unset distances correctly propagate through
    /// the triangle-inequality check and shortest-path composition.
    #[must_use]
    pub fn distance(&self, a: &T, b: &T) -> Tropical {
        self.distances
            .get(&(a.clone(), b.clone()))
            .copied()
            .unwrap_or_else(Tropical::zero)
    }

    /// Check the triangle inequality `d(x, z) ≤ d(x, y) + d(y, z)` over
    /// all triples `(x, y, z) ∈ objects³`.
    ///
    /// Returns `true` iff the inequality holds everywhere.
    ///
    /// # Complexity
    ///
    /// `O(n³)` where `n = self.objects.len()`. Intended for small finite
    /// spaces and test fixtures; not suitable for large metric spaces.
    #[must_use]
    #[allow(clippy::similar_names)]
    pub fn triangle_inequality_holds(&self) -> bool {
        for x in &self.objects {
            for y in &self.objects {
                for z in &self.objects {
                    let dxy = self.distance(x, y);
                    let dyz = self.distance(y, z);
                    let dxz = self.distance(x, z);
                    // Tropical multiplication = real addition — this is the
                    // defining property of the (min, +) semiring.
                    let sum = dxy * dyz;
                    // The metric inequality `d(x,z) ≤ d(x,y) + d(y,z)` is
                    // ordinary `≤` on the underlying `f64`, not the rig's
                    // `min` order.
                    if dxz.0 > sum.0 {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Build a Lawvere metric space from a [`UnitInterval`]-valued probability
    /// function via the `-ln π` embedding (see
    /// [`BaseChange<UnitInterval> for Tropical`](crate::rig::BaseChange)).
    ///
    /// Probabilities of `0` become `+∞` (unreachable); probabilities of `1`
    /// become `0` (self-identity distance).
    ///
    /// # Caller obligations
    ///
    /// To satisfy the Lawvere metric identity axiom (`d(x, x) = 0`), the
    /// caller must ensure `prob(x, x) = UnitInterval::new(1.0).unwrap()` for
    /// every object `x`. This constructor does not enforce the axiom — a
    /// closure that returns `prob(x, x) < 1.0` produces a structure where
    /// `d(x, x) > 0`, silently violating the axiom.
    /// [`triangle_inequality_holds`](Self::triangle_inequality_holds) checks
    /// only the triangle inequality; callers that want identity-axiom
    /// validation must assert it separately.
    ///
    /// # Iteration order
    ///
    /// The constructor iterates `objects × objects` in the `Vec<T>` order,
    /// not [`HashMap`] order — the `prob` closure sees a deterministic
    /// traversal.
    // Signature takes `Vec<T>` by value for symmetry with [`new`](Self::new),
    // which stores the list. Caller-side ergonomics: every test/example
    // constructs an owned `Vec<T>` and hands it over.
    #[allow(clippy::needless_pass_by_value)]
    pub fn from_unit_interval<F>(objects: Vec<T>, prob: F) -> Self
    where
        F: Fn(&T, &T) -> UnitInterval,
    {
        let mut space = Self::new(objects);
        // Iterate over the stored list — deterministic `Vec<T>` traversal.
        // Clone the outer handle once so we can mutate `space.distances`
        // from the inner loops without aliasing `space.objects`.
        let keys = space.objects.clone();
        for a in &keys {
            for b in &keys {
                let p = prob(a, b);
                let d = <Tropical as BaseChange<UnitInterval>>::base_change(p);
                space.distances.insert((a.clone(), b.clone()), d);
            }
        }
        space
    }
}

impl<T> EnrichedCategory<Tropical> for LawvereMetricSpace<T>
where
    T: Clone + Eq + Hash + 'static,
{
    type Object = T;

    fn hom(&self, a: &Self::Object, b: &Self::Object) -> Tropical {
        self.distance(a, b)
    }

    fn objects(&self) -> Box<dyn Iterator<Item = Self::Object> + '_> {
        Box::new(self.objects.iter().cloned())
    }
}
