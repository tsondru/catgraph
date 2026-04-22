//! F&S Thm 5.60 faithfulness tests for `S: SFG_R → Mat(R)` on bounded
//! enumerations of SFG expressions.
//!
//! # Status
//!
//! The 12 `thm_5_60_faithful_*` tests below remain `#[ignore]`'d as of v0.5.1.
//!
//! **What changed in v0.5.1 (but didn't close the gap):**
//! - `Presentation::eq_mod` now dispatches to a Knuth-Bendix-grade congruence-
//!   closure engine by default (see `catgraph_applied::prop::presentation::kb`).
//!   This correctly decides overlapping user equations — the v0.5.0 limitation
//!   that originally motivated the `#[ignore]`.
//! - The faithfulness harness now routes through `eq_mod` (not `normalize`),
//!   so the CC engine is actually consulted during enumeration.
//! - SMC Rule 9 (`Identity(m) ⊗ Identity(n) → Identity(m+n)`) was added to
//!   `apply_smc_rules`.
//!
//! **What still blocks re-enable:**
//! The SMC pre-pass in `apply_smc_rules` is a one-pass bottom-up rewriter. It
//! correctly canonicalizes terms where interchange applies at the given
//! factoring, but cannot re-associate to discover interchange opportunities.
//! Example witness: `ε ⊗ (σ_{1,1} ⊗ id_1)` vs `(ε ⊗ id_3) ; (σ_{1,1} ⊗ id_1)`
//! — these are equal by SMC coherence, but distinguishing them requires
//! rebalancing the outer tensor, re-associating, and then applying
//! interchange. Closing this requires Joyal-Street string-diagram normal form.
//!
//! Scheduled for v0.5.2.

use catgraph_applied::{
    graphical_linalg::{matr_presentation, verify_sfg_to_mat_is_full_and_faithful},
    rig::{BoolRig, F64Rig, Rig, Tropical, UnitInterval},
    sfg::SignalFlowGraph,
    sfg_to_mat::sfg_to_mat,
};

// ---- Smoke tests (always active): the presentation builds across all rigs ----

#[test]
fn matr_presentation_builds_bool() {
    let samples = vec![BoolRig(false), BoolRig(true)];
    matr_presentation::<BoolRig>(&samples).unwrap();
}

#[test]
fn matr_presentation_builds_f64() {
    let samples = vec![F64Rig(0.0), F64Rig(1.0), F64Rig(2.0)];
    matr_presentation::<F64Rig>(&samples).unwrap();
}

#[test]
fn matr_presentation_builds_tropical() {
    let samples = vec![Tropical(f64::INFINITY), Tropical(0.0), Tropical(1.0)];
    matr_presentation::<Tropical>(&samples).unwrap();
}

#[test]
fn matr_presentation_builds_unit_interval() {
    let samples = vec![
        UnitInterval::new(0.0).unwrap(),
        UnitInterval::new(0.5).unwrap(),
        UnitInterval::new(1.0).unwrap(),
    ];
    matr_presentation::<UnitInterval>(&samples).unwrap();
}

const IGNORE_REASON: &str =
    "Thm 5.60 syntactic faithfulness: Presentation normalizer is structural, not full \
     congruence closure; yields false-negative eq_mod. Reactivate when KB-completion \
     or a matrix-backed quotient is wired up.";

fn witness_debug<R>(
    report: &catgraph_applied::graphical_linalg::FaithfulnessReport<R>,
) -> Option<(String, String)>
where
    R: catgraph_applied::rig::Rig + std::fmt::Debug + Eq + std::hash::Hash + 'static,
{
    report.witnesses.first().map(|(a, b)| {
        (
            format!("{:?}", a.as_prop_expr()),
            format!("{:?}", b.as_prop_expr()),
        )
    })
}

// ---- BoolRig × {2, 3, 4} ----

#[test]
#[ignore = "Thm 5.60 syntactic faithfulness; see module docstring"]
fn thm_5_60_faithful_bool_depth_2() {
    let samples = vec![BoolRig(false), BoolRig(true)];
    let report = verify_sfg_to_mat_is_full_and_faithful::<BoolRig>(2, &samples).unwrap();
    assert_eq!(
        report.collisions_under_s, 0,
        "BoolRig depth 2: {} expressions, {} collisions; first witness: {:?}. {IGNORE_REASON}",
        report.expressions_checked,
        report.collisions_under_s,
        witness_debug(&report),
    );
}

#[test]
#[ignore = "Thm 5.60 syntactic faithfulness; see module docstring"]
fn thm_5_60_faithful_bool_depth_3() {
    let samples = vec![BoolRig(false), BoolRig(true)];
    let report = verify_sfg_to_mat_is_full_and_faithful::<BoolRig>(3, &samples).unwrap();
    assert_eq!(report.collisions_under_s, 0);
}

#[test]
#[ignore = "Thm 5.60 syntactic faithfulness; see module docstring"]
fn thm_5_60_faithful_bool_depth_4() {
    let samples = vec![BoolRig(false), BoolRig(true)];
    let report = verify_sfg_to_mat_is_full_and_faithful::<BoolRig>(4, &samples).unwrap();
    assert_eq!(report.collisions_under_s, 0);
}

// ---- UnitInterval × {2, 3, 4} ----

#[test]
#[ignore = "Thm 5.60 syntactic faithfulness; see module docstring"]
fn thm_5_60_faithful_unit_interval_depth_2() {
    let samples = vec![
        UnitInterval::new(0.0).unwrap(),
        UnitInterval::new(0.5).unwrap(),
        UnitInterval::new(1.0).unwrap(),
    ];
    let report =
        verify_sfg_to_mat_is_full_and_faithful::<UnitInterval>(2, &samples).unwrap();
    assert_eq!(
        report.collisions_under_s, 0,
        "UnitInterval depth 2: {} collisions; witness {:?}",
        report.collisions_under_s,
        witness_debug(&report),
    );
}

#[test]
#[ignore = "Thm 5.60 syntactic faithfulness; see module docstring"]
fn thm_5_60_faithful_unit_interval_depth_3() {
    let samples = vec![
        UnitInterval::new(0.0).unwrap(),
        UnitInterval::new(0.5).unwrap(),
        UnitInterval::new(1.0).unwrap(),
    ];
    let report =
        verify_sfg_to_mat_is_full_and_faithful::<UnitInterval>(3, &samples).unwrap();
    assert_eq!(report.collisions_under_s, 0);
}

#[test]
#[ignore = "Thm 5.60 syntactic faithfulness; see module docstring"]
fn thm_5_60_faithful_unit_interval_depth_4() {
    let samples = vec![
        UnitInterval::new(0.0).unwrap(),
        UnitInterval::new(0.5).unwrap(),
        UnitInterval::new(1.0).unwrap(),
    ];
    let report =
        verify_sfg_to_mat_is_full_and_faithful::<UnitInterval>(4, &samples).unwrap();
    assert_eq!(report.collisions_under_s, 0);
}

// ---- Tropical × {2, 3, 4} ----

#[test]
#[ignore = "Thm 5.60 syntactic faithfulness; see module docstring"]
fn thm_5_60_faithful_tropical_depth_2() {
    let samples = vec![
        Tropical(f64::INFINITY),
        Tropical(0.0),
        Tropical(1.0),
        Tropical(2.0),
    ];
    let report =
        verify_sfg_to_mat_is_full_and_faithful::<Tropical>(2, &samples).unwrap();
    assert_eq!(
        report.collisions_under_s, 0,
        "Tropical depth 2: {} collisions; witness {:?}",
        report.collisions_under_s,
        witness_debug(&report),
    );
}

#[test]
#[ignore = "Thm 5.60 syntactic faithfulness; see module docstring"]
fn thm_5_60_faithful_tropical_depth_3() {
    let samples = vec![
        Tropical(f64::INFINITY),
        Tropical(0.0),
        Tropical(1.0),
        Tropical(2.0),
    ];
    let report =
        verify_sfg_to_mat_is_full_and_faithful::<Tropical>(3, &samples).unwrap();
    assert_eq!(report.collisions_under_s, 0);
}

#[test]
#[ignore = "Thm 5.60 syntactic faithfulness; see module docstring"]
fn thm_5_60_faithful_tropical_depth_4() {
    let samples = vec![
        Tropical(f64::INFINITY),
        Tropical(0.0),
        Tropical(1.0),
        Tropical(2.0),
    ];
    let report =
        verify_sfg_to_mat_is_full_and_faithful::<Tropical>(4, &samples).unwrap();
    assert_eq!(report.collisions_under_s, 0);
}

// ---- F64Rig × {2, 3, 4} ----

#[test]
#[ignore = "Thm 5.60 syntactic faithfulness; see module docstring"]
fn thm_5_60_faithful_f64_depth_2() {
    let samples = vec![F64Rig(0.0), F64Rig(1.0), F64Rig(2.0), F64Rig(-1.0)];
    let report = verify_sfg_to_mat_is_full_and_faithful::<F64Rig>(2, &samples).unwrap();
    assert_eq!(
        report.collisions_under_s, 0,
        "F64Rig depth 2: {} collisions; witness {:?}",
        report.collisions_under_s,
        witness_debug(&report),
    );
}

#[test]
#[ignore = "Thm 5.60 syntactic faithfulness; see module docstring"]
fn thm_5_60_faithful_f64_depth_3() {
    let samples = vec![F64Rig(0.0), F64Rig(1.0), F64Rig(2.0), F64Rig(-1.0)];
    let report = verify_sfg_to_mat_is_full_and_faithful::<F64Rig>(3, &samples).unwrap();
    assert_eq!(report.collisions_under_s, 0);
}

#[test]
#[ignore = "Thm 5.60 syntactic faithfulness; see module docstring"]
fn thm_5_60_faithful_f64_depth_4() {
    let samples = vec![F64Rig(0.0), F64Rig(1.0), F64Rig(2.0), F64Rig(-1.0)];
    let report = verify_sfg_to_mat_is_full_and_faithful::<F64Rig>(4, &samples).unwrap();
    assert_eq!(report.collisions_under_s, 0);
}

// ---- Thm 5.60 soundness: every equation in the presentation is a matrix equality under S ----

/// For each equation `(lhs, rhs)` in the Thm 5.60 presentation, verify that
/// `S(lhs) == S(rhs)` under `sfg_to_mat`. This is the SOUNDNESS direction
/// (S is well-defined on the quotient); the FAITHFULNESS direction (S is
/// injective on the quotient) is the harder direction and requires KB
/// completion — deferred to v0.5.1.
fn assert_soundness_for_rig<R>(rig_samples: &[R]) -> String
where
    R: Rig + std::fmt::Debug + Eq + std::hash::Hash + 'static,
{
    let presentation = matr_presentation::<R>(rig_samples)
        .expect("matr_presentation builds");

    let mut violations: Vec<String> = Vec::new();
    for (i, (lhs, rhs)) in presentation.equations().iter().enumerate() {
        let lhs_sfg = SignalFlowGraph::<R>::from_prop_expr(lhs.clone());
        let rhs_sfg = SignalFlowGraph::<R>::from_prop_expr(rhs.clone());

        let lhs_mat = sfg_to_mat(&lhs_sfg);
        let rhs_mat = sfg_to_mat(&rhs_sfg);

        match (lhs_mat, rhs_mat) {
            (Ok(a), Ok(b)) => {
                if a != b {
                    violations.push(format!(
                        "eq #{i}: sfg_to_mat(lhs) != sfg_to_mat(rhs)\n  lhs={lhs:?}\n  rhs={rhs:?}\n  S(lhs)={a:?}\n  S(rhs)={b:?}"
                    ));
                }
            }
            (e_a, e_b) => {
                violations.push(format!(
                    "eq #{i}: sfg_to_mat failed: lhs={e_a:?}, rhs={e_b:?}"
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Soundness violations: {}",
        violations.join("\n\n")
    );
    format!("{} equations sound under S", presentation.equations().len())
}

#[test]
fn thm_5_60_soundness_f64() {
    let samples = vec![F64Rig(0.0), F64Rig(1.0), F64Rig(2.0), F64Rig(-1.0)];
    let report = assert_soundness_for_rig::<F64Rig>(&samples);
    println!("F64Rig: {report}");
}

#[test]
fn thm_5_60_soundness_bool() {
    let samples = vec![BoolRig(false), BoolRig(true)];
    let report = assert_soundness_for_rig::<BoolRig>(&samples);
    println!("BoolRig: {report}");
}

#[test]
fn thm_5_60_soundness_unit_interval() {
    let samples = vec![
        UnitInterval::new(0.0).unwrap(),
        UnitInterval::new(0.5).unwrap(),
        UnitInterval::new(1.0).unwrap(),
    ];
    let report = assert_soundness_for_rig::<UnitInterval>(&samples);
    println!("UnitInterval: {report}");
}

#[test]
fn thm_5_60_soundness_tropical() {
    let samples = vec![
        Tropical(f64::INFINITY),
        Tropical(0.0),
        Tropical(1.0),
        Tropical(2.0),
    ];
    let report = assert_soundness_for_rig::<Tropical>(&samples);
    println!("Tropical: {report}");
}
