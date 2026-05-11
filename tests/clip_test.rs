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
fn test_line_fully_outside_opposite_sides() {
    // Line goes from far left to far right but above the box
    let result = clip_line(-5.0, 10.0, 10.0, 10.0, &[0.0, 5.0], &[0.0, 5.0]);
    assert!(result.is_none());
}
