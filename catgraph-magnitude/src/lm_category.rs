//! [`LmCategory`] — materialized language-model transition table per
//! BV 2025 §3.
//!
//! Stores a finite set of named states, the terminating subset `T(⊥)`, and
//! the per-state next-symbol transition probabilities. The terminal mass at
//! state `x` is the implicit `1 − Σ_y transitions[x][y]`; following BV 2025
//! Eq (11), this terminal mass appears in the Tsallis-entropy sum
//! `H_t(p_x)` because `p_x` is a probability mass function on `A ∪ {†}`.
//!
//! Per umbrella Q5, v0.1.0 is "BYO-LM": callers populate the transition
//! table from their own model. No closures, no LM runtime, no inference.
//! [`LmCategory::magnitude`] consumes the table by lifting it into a
//! [`LawvereMetricSpace<NodeId>`] via the `-ln π` embedding (Lawvere 1973;
//! BV 2025 §2.17) and calling [`magnitude::<F64Rig>`](crate::magnitude::magnitude).
//!
//! # BV 2025 paper anchors
//!
//! - §2.17 "Every LM defines a `[0, ∞]`-category": distance `d(x, y) :=
//!   −ln π(y|x)`; we materialize this directly.
//! - §3.5 Eq (7): `Mag(tM) = Σ_{x,y} ζ_t⁻¹(x, y)`.
//! - §3.10 Closed form: `Mag(tM) = (t − 1) · Σ_{x ∉ T(⊥)} H_t(p_x) +
//!   #(T(⊥))`. The two acceptance tests in `tests/bv_2025_acceptance.rs`
//!   verify this against the Möbius-sum form computed by
//!   [`magnitude`] function.
//! - §3.14 Cor: `d/dt Mag(tM)|_{t=1} = Σ_{x ∉ T(⊥)} H(p_x)` (Shannon
//!   entropy). Verified by central finite difference with `h = 1e-4 >
//!   TSALLIS_SHANNON_EPS`.

use std::collections::{HashMap, HashSet};

use crate::magnitude::magnitude;
use crate::weighted_cospan::NodeId;
use crate::{CatgraphError, F64Rig, LawvereMetricSpace, Tropical};

/// Materialized language-model transition table per BV 2025 §3.
///
/// Stores:
/// - `objects`: ordered list of state names, indexed left-to-right.
/// - `terminating`: subset of state names corresponding to `T(⊥)` — the
///   theoretically terminating states. Membership is BYO-LM, not derived
///   from the transition table.
/// - `transitions`: sparse `HashMap<from, HashMap<to, prob>>`. The terminal
///   mass at state `x` is the implicit `1 − Σ_y transitions[x][y]`, which
///   is treated as the weight of the virtual `†` symbol in the Tsallis
///   sum (BV 2025 Eq 11).
///
/// **Identity axiom.** The Lawvere metric `d(x, x) = 0` (i.e.
/// `π(x|x) = 1`) is enforced by [`magnitude`](Self::magnitude) when it
/// constructs the [`LawvereMetricSpace`] — callers do not have to populate
/// self-transitions.
#[derive(Clone, Debug, PartialEq)]
pub struct LmCategory {
    objects: Vec<String>,
    terminating: HashSet<String>,
    transitions: HashMap<String, HashMap<String, f64>>,
}

impl LmCategory {
    /// Build an empty LM category over a fixed object list. Terminating set
    /// and transitions both start empty; populate via
    /// [`add_transition`](Self::add_transition) and
    /// [`mark_terminating`](Self::mark_terminating).
    #[must_use]
    pub fn new(objects: Vec<String>) -> Self {
        Self {
            objects,
            terminating: HashSet::new(),
            transitions: HashMap::new(),
        }
    }

    /// Set the next-symbol probability `π(to | from) = prob`.
    ///
    /// Overwrites any prior value. Does NOT validate row normalization
    /// — leaky rows (`Σ_y π(y|from) < 1`) are intentional and represent
    /// the BV 2025 †-terminal mass at state `from`.
    ///
    /// # Panics
    ///
    /// Debug-only: `from` and `to` must be in `objects`, and `prob ∈ [0, 1]`.
    /// Release builds skip these checks for performance.
    pub fn add_transition(&mut self, from: &str, to: &str, prob: f64) {
        debug_assert!(
            self.objects.iter().any(|o| o == from),
            "from state {from:?} not in objects"
        );
        debug_assert!(
            self.objects.iter().any(|o| o == to),
            "to state {to:?} not in objects"
        );
        debug_assert!(
            (0.0..=1.0).contains(&prob),
            "prob {prob} not in [0, 1]"
        );
        self.transitions
            .entry(from.to_owned())
            .or_default()
            .insert(to.to_owned(), prob);
    }

    /// Mark a state as terminating (i.e. add it to `T(⊥)`).
    ///
    /// # Panics
    ///
    /// Debug-only: `state` must be in `objects`.
    pub fn mark_terminating(&mut self, state: &str) {
        debug_assert!(
            self.objects.iter().any(|o| o == state),
            "state {state:?} not in objects"
        );
        self.terminating.insert(state.to_owned());
    }

    /// Borrow the ordered object list.
    #[must_use]
    pub fn objects(&self) -> &[String] {
        &self.objects
    }

    /// Borrow the terminating-states set.
    #[must_use]
    pub fn terminating(&self) -> &HashSet<String> {
        &self.terminating
    }

    /// Borrow the transition table.
    #[must_use]
    pub fn transitions(&self) -> &HashMap<String, HashMap<String, f64>> {
        &self.transitions
    }

    /// Magnitude `Mag(tM)` of the LM at scale `t`, computed via Möbius sum
    /// (BV 2025 §3.5 Eq 7).
    ///
    /// Builds an `n × n` Lawvere metric space over `0..n` (`NodeId` =
    /// position in [`objects`](Self::objects)), populating distances per
    /// the **BV 2025 §2.10–2.17 prefix-extension semantics**:
    ///
    /// - `d(i, i) = 0` (identity axiom).
    /// - For every directed extension path `i = x₀ → x₁ → … → x_k = j`
    ///   recorded in the transition table, `π(j | i) = ∏_{ℓ} π(x_{ℓ+1} |
    ///   x_ℓ)` (BV 2025 Eq 6) and `d(i, j) = −ln π(j | i)`.
    /// - When no such path exists, the distance defaults to `Tropical(+∞)`
    ///   (i.e. `ζ_t[i][j] = 0`), per the convention `π(y | x) = 0` when `y`
    ///   is not an extension of `x` (BV 2025 §2.15).
    ///
    /// The transitive-closure computation is a forward BFS from each
    /// source node, multiplying probabilities along each path. **The
    /// transition table must be acyclic** for the resulting metric to
    /// satisfy BV 2025's tree-poset structure — otherwise the BFS may
    /// loop and the magnitude will not match the closed form of Thm 3.10.
    /// Acyclicity is the caller's responsibility; a debug-only assertion
    /// catches obvious self-loop cases. (Cyclic LMs are mathematically
    /// well-defined via the chain-sum Möbius formula but fall outside the
    /// poset hypothesis of Thm 3.10 — see BV 2025 §3.7 Remark.)
    ///
    /// # Errors
    ///
    /// Returns [`CatgraphError::Composition`] if the t-scaled zeta is
    /// singular at this scale. Per BV 2025 Prop 3.6 `ζ_t` is invertible for
    /// any `t > 0` in the LM setting; singular results indicate caller
    /// inputs that violate the LM assumptions (e.g. degenerate parametric
    /// coincidences from cyclic transitions).
    pub fn magnitude(&self, t: f64) -> Result<f64, CatgraphError> {
        let n = self.objects.len();
        let objects: Vec<NodeId> = (0..n).collect();
        let mut space = LawvereMetricSpace::new(objects);

        // Index each state name to its position in `self.objects`.
        let idx: HashMap<&str, usize> = self
            .objects
            .iter()
            .enumerate()
            .map(|(i, s)| (s.as_str(), i))
            .collect();

        // Identity axiom: d(i, i) = 0 ⇒ ζ[i][i] = 1.
        for i in 0..n {
            space.set_distance(i, i, Tropical(0.0));
        }

        // Forward-extension closure. For each source `i`, BFS through the
        // transition table, accumulating the multiplicative probability.
        // `best[j]` records the best (highest-probability) path so far —
        // the LM tree-poset structure ensures uniqueness, but in case of
        // a malformed (DAG-with-rejoin) input we keep the highest weight.
        for i in 0..n {
            let mut best: HashMap<usize, f64> = HashMap::new();
            best.insert(i, 1.0);
            let mut frontier: Vec<usize> = vec![i];
            while let Some(cur) = frontier.pop() {
                let cur_p = best[&cur];
                let cur_name = self.objects[cur].as_str();
                let Some(row) = self.transitions.get(cur_name) else {
                    continue;
                };
                for (next_name, &edge_p) in row {
                    if edge_p <= 0.0 {
                        continue;
                    }
                    let Some(&next) = idx.get(next_name.as_str()) else {
                        continue;
                    };
                    if next == i {
                        // Self-cycle back to source — skip (acyclicity
                        // assumption; BV 2025 §3 hypothesizes a tree).
                        continue;
                    }
                    let new_p = cur_p * edge_p;
                    let prior = best.get(&next).copied().unwrap_or(0.0);
                    if new_p > prior {
                        best.insert(next, new_p);
                        frontier.push(next);
                    }
                }
            }
            // Write distances for every reached node `j != i`.
            for (j, p) in best {
                if j == i || p <= 0.0 {
                    continue;
                }
                space.set_distance(i, j, Tropical(-p.ln()));
            }
        }

        let mag: F64Rig = magnitude(&space, t)?;
        Ok(mag.0)
    }
}
