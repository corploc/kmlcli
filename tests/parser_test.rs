use kmlcli::parser::parse_file;
use std::path::Path;

#[test]
fn test_parse_simple_kml() {
    let doc = parse_file(Path::new("tests/fixtures/simple.kml")).unwrap();
    assert_eq!(doc.name.as_deref(), Some("Test Document"));
    assert_eq!(doc.features.len(), 2); // 1 folder + 1 placemark
}

#[test]
fn test_parse_multi_kml_with_styles() {
    let doc = parse_file(Path::new("tests/fixtures/multi.kml")).unwrap();
    assert_eq!(doc.name.as_deref(), Some("Multi Test"));
    assert!(doc.styles.contains_key("redLine"));
    let style = &doc.styles["redLine"];
    assert_eq!(style.line_color.as_deref(), Some("ff0000ff"));
}

#[test]
fn test_parse_empty_kml() {
    let doc = parse_file(Path::new("tests/fixtures/empty.kml")).unwrap();
    assert_eq!(doc.name.as_deref(), Some("Empty"));
    assert!(doc.features.is_empty());
}

#[test]
fn test_parse_nonexistent_file() {
    let result = parse_file(Path::new("tests/fixtures/nope.kml"));
    assert!(result.is_err());
}
