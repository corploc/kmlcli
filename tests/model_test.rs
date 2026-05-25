use kmlcli::model::*;
use std::collections::HashMap;

fn make_doc() -> KmlDocument {
    KmlDocument {
        name: Some("Test".to_string()),
        features: vec![
            Feature::Folder {
                name: "Folder 1".to_string(),
                features: vec![Feature::Placemark {
                    name: "Point A".to_string(),
                    geometry: Some(Geometry::Point(Coord {
                        lon: -1.6778,
                        lat: 48.1173,
                        alt: None,
                    })),
                    style_id: None,
                    description: None,
                }],
            },
            Feature::Placemark {
                name: "Line B".to_string(),
                geometry: Some(Geometry::LineString(vec![
                    Coord {
                        lon: -1.6778,
                        lat: 48.1173,
                        alt: None,
                    },
                    Coord {
                        lon: -1.6800,
                        lat: 48.1200,
                        alt: None,
                    },
                ])),
                style_id: None,
                description: None,
            },
        ],
        styles: HashMap::new(),
        style_maps: HashMap::new(),
    }
}

#[test]
fn test_all_coords_returns_all_points() {
    let doc = make_doc();
    let coords = doc.all_coords();
    assert_eq!(coords.len(), 3);
}

#[test]
fn test_bounding_box_covers_all_coords() {
    let doc = make_doc();
    let bbox = doc.bounding_box().unwrap();
    assert!(bbox.min_lon <= -1.6800);
    assert!(bbox.max_lon >= -1.6778);
    assert!(bbox.min_lat <= 48.1173);
    assert!(bbox.max_lat >= 48.1200);
}

#[test]
fn test_bounding_box_single_point() {
    let doc = KmlDocument {
        name: None,
        features: vec![Feature::Placemark {
            name: "Solo".to_string(),
            geometry: Some(Geometry::Point(Coord {
                lon: 2.0,
                lat: 48.0,
                alt: None,
            })),
            style_id: None,
            description: None,
        }],
        styles: HashMap::new(),
        style_maps: HashMap::new(),
    };
    let bbox = doc.bounding_box().unwrap();
    assert!(bbox.max_lon > bbox.min_lon);
    assert!(bbox.max_lat > bbox.min_lat);
}

#[test]
fn test_bounding_box_empty() {
    let doc = KmlDocument {
        name: None,
        features: vec![],
        styles: HashMap::new(),
        style_maps: HashMap::new(),
    };
    assert!(doc.bounding_box().is_none());
}

#[test]
fn test_flatten_returns_all_features_with_paths() {
    let doc = make_doc();
    let flat = doc.flatten();
    assert_eq!(flat.len(), 3);
    assert_eq!(flat[0].0, vec![0]);
    assert_eq!(flat[1].0, vec![0, 0]);
    assert_eq!(flat[2].0, vec![1]);
}
