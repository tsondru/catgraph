//! [`LmCategory`] — materialized language-model transition table per
//! BV 2025 §3. Phase 6A.3 stub.
//!
//! Populated in the 6A.3 commit:
//! - `objects: Vec<String>`, `terminating: HashSet<String>`, `transitions: HashMap<String, HashMap<String, f64>>`
//! - `new`, `add_transition`, `mark_terminating`, `magnitude(&self, t: f64) -> f64`
//! - BV 2025 Thm 3.10 closed-form acceptance test
//! - BV 2025 Cor 3.14 Shannon recovery via finite difference (h = 1e-4 central)

// Stub — populated in Phase 6A.3.
