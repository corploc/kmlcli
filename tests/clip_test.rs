use kmlcli::clip::clip_line;

#[test]
fn test_line_fully_inside() {
    let result = clip_line(1.0, 1.0, 3.0, 3.0, &[0.0, 5.0], &[0.0, 5.0]);
    assert_eq!(result, Some((1.0, 1.0, 3.0, 3.0)));
}

#[test]
fn test_line_fully_outside_same_side() {
    let result = clip_line(-3.0, -2.0, -1.0, -1.0, &[0.0, 5.0], &[0.0, 5.0]);
    assert!(result.is_none());
}

#[test]
fn test_line_crossing_left_boundary() {
    let result = clip_line(-1.0, 2.5, 2.0, 2.5, &[0.0, 5.0], &[0.0, 5.0]);
    let (x1, y1, x2, y2) = result.unwrap();
    assert!((x1 - 0.0).abs() < 0.01, "x1={x1} should be ~0.0");
    assert!((y1 - 2.5).abs() < 0.01);
    assert!((x2 - 2.0).abs() < 0.01);
    assert!((y2 - 2.5).abs() < 0.01);
}

#[test]
fn test_line_crossing_both_boundaries() {
    let result = clip_line(-1.0, 2.5, 6.0, 2.5, &[0.0, 5.0], &[0.0, 5.0]);
    let (x1, _y1, x2, _y2) = result.unwrap();
    assert!((x1 - 0.0).abs() < 0.01);
    assert!((x2 - 5.0).abs() < 0.01);
}

#[test]
fn clip_horizontal_segment_crossing_box() {
    // Horizontal segment crossing the box left-to-right at y=0.5.
    // Pins the invariant that dy=0 with TOP/BOTTOM in code_out cannot occur
    // (both endpoints would share the same y, so code1 & code2 would short-circuit).
    let result = clip_line(-1.0, 0.5, 2.0, 0.5, &[0.0, 1.0], &[0.0, 1.0]);
    let (x1, y1, x2, y2) = result.unwrap();
    assert!((x1 - 0.0).abs() < 1e-9, "x1={x1}");
    assert!((y1 - 0.5).abs() < 1e-9);
    assert!((x2 - 1.0).abs() < 1e-9, "x2={x2}");
    assert!((y2 - 0.5).abs() < 1e-9);
}

#[test]
fn clip_vertical_segment_crossing_box() {
    let result = clip_line(0.5, -1.0, 0.5, 2.0, &[0.0, 1.0], &[0.0, 1.0]);
    let (x1, y1, x2, y2) = result.unwrap();
    assert!((x1 - 0.5).abs() < 1e-9);
    assert!((y1 - 0.0).abs() < 1e-9, "y1={y1}");
    assert!((x2 - 0.5).abs() < 1e-9);
    assert!((y2 - 1.0).abs() < 1e-9, "y2={y2}");
}

#[test]
fn clip_horizontal_segment_above_box() {
    // Both endpoints above — must return None, no division by dy=0.
    let result = clip_line(-1.0, 2.0, 2.0, 2.0, &[0.0, 1.0], &[0.0, 1.0]);
    assert_eq!(result, None);
}

#[test]
fn clip_vertical_segment_left_of_box() {
    let result = clip_line(-1.0, -1.0, -1.0, 2.0, &[0.0, 1.0], &[0.0, 1.0]);
    assert_eq!(result, None);
}

#[test]
fn test_line_fully_outside_opposite_sides() {
    // Line goes from far left to far right but above the box
    let result = clip_line(-5.0, 10.0, 10.0, 10.0, &[0.0, 5.0], &[0.0, 5.0]);
    assert!(result.is_none());
}
