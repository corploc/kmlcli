use kmlcli::tiles::decode::DecodedFeature;
use kmlcli::tiles::proto::GeomType;
use kmlcli::tiles::render::{TileLabel, dedup_labels, prerender_tile};
use std::collections::HashMap;

fn make_feature(
    layer: &str,
    class: &str,
    rings: Vec<Vec<(f64, f64)>>,
    geom_type: GeomType,
    name: Option<&str>,
) -> DecodedFeature {
    let mut properties = HashMap::new();
    if !class.is_empty() {
        properties.insert("class".to_string(), class.to_string());
    }
    if let Some(n) = name {
        properties.insert("name".to_string(), n.to_string());
    }
    DecodedFeature {
        layer: layer.to_string(),
        geom_type,
        rings,
        properties,
    }
}

#[test]
fn test_prerender_water_polygon_produces_segments() {
    let features = vec![make_feature(
        "water",
        "",
        vec![vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 0.0)]],
        GeomType::Polygon,
        None,
    )];
    let rendered = prerender_tile(&features);
    assert!(!rendered.segments.is_empty());
    // 3 segments for a triangle (3 windows of 2)
    assert_eq!(rendered.segments.len(), 3);
}

#[test]
fn test_prerender_skips_unknown_layers() {
    let features = vec![make_feature(
        "unknown_layer",
        "",
        vec![vec![(0.0, 0.0), (1.0, 1.0)]],
        GeomType::LineString,
        None,
    )];
    let rendered = prerender_tile(&features);
    assert!(rendered.segments.is_empty());
}

#[test]
fn test_prerender_skips_micro_segments() {
    let features = vec![make_feature(
        "water",
        "",
        vec![vec![(0.0, 0.0), (0.000001, 0.000001)]],
        GeomType::LineString,
        None,
    )];
    let rendered = prerender_tile(&features);
    assert!(
        rendered.segments.is_empty(),
        "micro segment should be filtered"
    );
}

#[test]
fn test_prerender_extracts_place_labels() {
    let features = vec![make_feature(
        "place",
        "city",
        vec![vec![(2.35, 48.86)]],
        GeomType::Point,
        Some("Paris"),
    )];
    let rendered = prerender_tile(&features);
    assert_eq!(rendered.labels.len(), 1);
    assert_eq!(rendered.labels[0].text, "Paris");
    assert_eq!(rendered.labels[0].min_zoom, 6);
}

#[test]
fn test_prerender_skips_labels_without_name() {
    let features = vec![make_feature(
        "place",
        "city",
        vec![vec![(2.35, 48.86)]],
        GeomType::Point,
        None,
    )];
    let rendered = prerender_tile(&features);
    assert!(rendered.labels.is_empty());
}

#[test]
fn test_prerender_road_colors_differ_by_class() {
    let motorway = make_feature(
        "transportation",
        "motorway",
        vec![vec![(0.0, 0.0), (1.0, 1.0)]],
        GeomType::LineString,
        None,
    );
    let tertiary = make_feature(
        "transportation",
        "tertiary",
        vec![vec![(0.0, 0.0), (1.0, 1.0)]],
        GeomType::LineString,
        None,
    );
    let r1 = prerender_tile(&[motorway]);
    let r2 = prerender_tile(&[tertiary]);
    assert_ne!(r1.segments[0].color, r2.segments[0].color);
}

#[test]
fn test_dedup_removes_same_text() {
    let mut labels = vec![
        TileLabel {
            x: 0.0,
            y: 0.0,
            text: "Paris".to_string(),
            color: ratatui::style::Color::White,
            min_zoom: 0,
        },
        TileLabel {
            x: 0.1,
            y: 0.1,
            text: "Paris".to_string(),
            color: ratatui::style::Color::White,
            min_zoom: 0,
        },
        TileLabel {
            x: 5.0,
            y: 5.0,
            text: "London".to_string(),
            color: ratatui::style::Color::White,
            min_zoom: 0,
        },
    ];
    dedup_labels(&mut labels);
    assert_eq!(labels.len(), 2);
    let names: Vec<&str> = labels.iter().map(|l| l.text.as_str()).collect();
    assert!(names.contains(&"Paris"));
    assert!(names.contains(&"London"));
}

#[test]
fn test_dedup_keeps_different_texts() {
    let mut labels = vec![
        TileLabel {
            x: 0.0,
            y: 0.0,
            text: "A".to_string(),
            color: ratatui::style::Color::White,
            min_zoom: 0,
        },
        TileLabel {
            x: 0.0,
            y: 0.0,
            text: "B".to_string(),
            color: ratatui::style::Color::White,
            min_zoom: 0,
        },
        TileLabel {
            x: 0.0,
            y: 0.0,
            text: "C".to_string(),
            color: ratatui::style::Color::White,
            min_zoom: 0,
        },
    ];
    dedup_labels(&mut labels);
    assert_eq!(labels.len(), 3);
}
