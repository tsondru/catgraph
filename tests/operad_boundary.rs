use catgraph::e1_operad::E1;
use catgraph::e2_operad::E2;
use catgraph::errors::CatgraphError;
use catgraph::operadic::Operadic;

#[test]
fn e1_exact_unit_interval() {
    let e1 = E1::new(vec![(0.0, 1.0)], true);
    assert!(e1.is_ok());
    assert_eq!(e1.unwrap().extract_sub_intervals(), vec![(0.0, 1.0)]);
}

#[test]
fn e1_barely_invalid() {
    // -0.01 is well beyond epsilon tolerance
    let result = E1::new(vec![(-0.01, 0.5)], true);
    assert!(result.is_err());
    match result.unwrap_err() {
        CatgraphError::Operadic(msg) => assert!(msg.contains("below 0"), "got: {msg}"),
        other => panic!("Expected Operadic error, got: {other:?}"),
    }
}

#[test]
fn e1_touching_intervals() {
    // Two intervals that touch at 0.5 — within epsilon tolerance
    let result = E1::new(vec![(0.0, 0.5), (0.5, 1.0)], true);
    assert!(result.is_ok(), "Touching intervals should be accepted: {result:?}");
}

#[test]
fn e1_overlapping_intervals() {
    // Intervals [0.0, 0.6) and [0.5, 1.0) genuinely overlap by 0.1
    let result = E1::new(vec![(0.0, 0.6), (0.5, 1.0)], true);
    assert!(result.is_err());
    match result.unwrap_err() {
        CatgraphError::Operadic(msg) => assert!(msg.contains("overlap"), "got: {msg}"),
        other => panic!("Expected Operadic error, got: {other:?}"),
    }
}

#[test]
fn e2_disk_at_boundary() {
    // Disk at (0.5, 0.0) with radius 0.5 — edge touches unit disk boundary
    // dist(origin, center) + radius = 0.5 + 0.5 = 1.0
    let result = E2::new(vec![(0, (0.5, 0.0), 0.5)], true);
    assert!(result.is_ok(), "Disk touching boundary should be accepted: {result:?}");
}

#[test]
fn e2_disk_outside() {
    // Disk at (0.8, 0.0) with radius 0.5 — extends past unit disk
    // dist(origin, center) + radius = 0.8 + 0.5 = 1.3 > 1.0
    let result = E2::new(vec![(0, (0.8, 0.0), 0.5)], true);
    assert!(result.is_err());
    match result.unwrap_err() {
        CatgraphError::Operadic(msg) => assert!(msg.contains("not contained"), "got: {msg}"),
        other => panic!("Expected Operadic error, got: {other:?}"),
    }
}

#[test]
fn e2_touching_disks() {
    // Two disks whose edges touch (closeness ~0) — should NOT be detected as overlap
    // Disk 1: center (-0.4, 0.0), radius 0.2
    // Disk 2: center (0.4, 0.0), radius 0.2
    // Distance between centers = 0.8, sum of radii = 0.4, closeness = 0.4
    // Actually let's use disks that truly touch:
    // Disk 1: center (-0.25, 0.0), radius 0.25
    // Disk 2: center (0.25, 0.0), radius 0.25
    // Distance between centers = 0.5, sum of radii = 0.5, closeness = 0.0
    let result = E2::new(
        vec![
            (0, (-0.25, 0.0), 0.25),
            (1, (0.25, 0.0), 0.25),
        ],
        true,
    );
    assert!(result.is_ok(), "Touching disks (closeness ~0) should not be overlap: {result:?}");
}

#[test]
fn e1_to_e2_embedding() {
    // Create E1 with 3 non-overlapping intervals
    let e1 = E1::new(vec![(0.0, 0.2), (0.3, 0.5), (0.7, 0.9)], true).unwrap();
    let e2: E2<usize> = E2::from_e1_config(e1, |idx| idx);

    // Verify all disks fit in the unit disk by re-constructing with overlap check
    let circles = e2.extract_sub_circles();
    assert_eq!(circles.len(), 3);
    let result = E2::new(circles, true);
    assert!(result.is_ok(), "E1-to-E2 embedding should produce valid E2: {result:?}");
}

#[test]
fn e1_substitution_validity() {
    // Compose two valid E1s via operadic_substitution, verify result is valid
    let mut outer = E1::new(vec![(0.1, 0.4), (0.6, 0.9)], true).unwrap();
    let inner = E1::new(vec![(0.1, 0.4), (0.6, 0.9)], true).unwrap();

    let result = outer.operadic_substitution(0, inner);
    assert!(result.is_ok(), "Substitution should succeed: {result:?}");

    // The resulting sub-intervals should still be valid within [0,1]
    let intervals = outer.extract_sub_intervals();
    for (a, b) in &intervals {
        assert!(*a >= 0.0, "interval start {a} should be >= 0");
        assert!(*b <= 1.0, "interval end {b} should be <= 1");
        assert!(*a < *b, "interval ({a}, {b}) should have positive width");
    }

    // Verify we can construct a new E1 from these intervals
    let reconstructed = E1::new(intervals, true);
    assert!(reconstructed.is_ok(), "Substitution result should be reconstructible: {reconstructed:?}");
}

#[test]
fn e2_substitution_validity() {
    // Compose two valid E2s via operadic_substitution, verify result is valid
    let mut outer: E2<i32> = E2::new(
        vec![
            (0, (-0.5, 0.0), 0.3),
            (1, (0.5, 0.0), 0.3),
        ],
        true,
    ).unwrap();

    let inner: E2<i32> = E2::new(
        vec![
            (2, (0.0, 0.0), 0.5),
        ],
        false,
    ).unwrap();

    let result = outer.operadic_substitution(0, inner);
    assert!(result.is_ok(), "E2 substitution should succeed: {result:?}");

    // Verify the result has 2 circles (removed 0, added 2, kept 1)
    let circles = outer.extract_sub_circles();
    assert_eq!(circles.len(), 2);

    // Verify we can construct a new E2 from these circles (geometry valid)
    let reconstructed = E2::new(circles, false);
    assert!(reconstructed.is_ok(), "E2 substitution result should be reconstructible: {reconstructed:?}");
}
