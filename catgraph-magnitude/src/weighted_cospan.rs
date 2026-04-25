//! [`WeightedCospan<Λ, Q>`] — `catgraph::Cospan<Λ>` decorated with per-edge
//! weights in a rig `Q`. Phase 6A.1 stub.
//!
//! Populated in the 6A.1 commit:
//! - newtype wrapper around `Cospan<Λ>` + `HashMap<(NodeId, NodeId), Q>`
//! - constructors `from_cospan_uniform`, `from_cospan_with_weights`
//! - `weight` / `set_weight` accessors
//! - `into_metric_space` bridge for `Q = UnitInterval` via `-ln π`
//! - type aliases `ProbCospan<Λ>` / `TropCospan<Λ>`

// Stub — populated in Phase 6A.1.
