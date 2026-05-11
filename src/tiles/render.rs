use ratatui::style::Color;

use super::decode::DecodedFeature;
use super::proto::GeomType;
use crate::model::Coord;
use crate::projection::Viewport;

pub struct TileSegment {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub color: Color,
}

pub struct TileLabel {
    pub x: f64,
    pub y: f64,
    pub text: String,
    pub color: Color,
}

pub fn render_tile_features(features: &[DecodedFeature], viewport: &Viewport) -> Vec<TileSegment> {
    let mut segments = Vec::new();

    for feature in features {
        let color = match road_color(&feature.layer, &feature.properties)
            .or_else(|| layer_color(&feature.layer, &feature.properties))
        {
            Some(c) => c,
            None => continue,
        };

        match feature.geom_type {
            GeomType::LineString | GeomType::Polygon => {
                for ring in &feature.rings {
                    for window in ring.windows(2) {
                        let coord1 = Coord {
                            lon: window[0].0,
                            lat: window[0].1,
                            alt: None,
                        };
                        let coord2 = Coord {
                            lon: window[1].0,
                            lat: window[1].1,
                            alt: None,
                        };
                        let (x1, y1) = viewport.project_for_canvas(&coord1);
                        let (x2, y2) = viewport.project_for_canvas(&coord2);
                        segments.push(TileSegment {
                            x1,
                            y1,
                            x2,
                            y2,
                            color,
                        });
                    }
                }
            }
            GeomType::Point | GeomType::Unknown => {}
        }
    }

    segments
}

/// Extract labels from decoded tile features.
/// Filters by zoom level: low zoom = countries only, higher zoom = more detail.
pub fn render_tile_labels(
    features: &[DecodedFeature],
    viewport: &Viewport,
    zoom: u32,
) -> Vec<TileLabel> {
    let mut labels = Vec::new();

    for feature in features {
        let name = match feature.properties.get("name") {
            Some(n) if !n.is_empty() => n,
            _ => continue,
        };

        let color = match label_color(&feature.layer, &feature.properties, zoom) {
            Some(c) => c,
            None => continue,
        };

        // Use first point of first ring as label position
        let (lon, lat) = match feature.rings.first().and_then(|r| r.first()) {
            Some(&pt) => pt,
            None => continue,
        };

        let coord = Coord {
            lon,
            lat,
            alt: None,
        };
        let (x, y) = viewport.project_for_canvas(&coord);

        labels.push(TileLabel {
            x,
            y,
            text: name.clone(),
            color,
        });
    }

    labels
}

/// Deduplicate labels that have the same text and are close together.
/// O(n log n) — sort then single-pass retain.
pub fn dedup_labels(labels: &mut Vec<TileLabel>) {
    labels.sort_by(|a, b| a.text.cmp(&b.text));

    // Single pass: keep first occurrence of each text+proximity group
    let mut kept: Vec<TileLabel> = Vec::with_capacity(labels.len());
    for label in labels.drain(..) {
        let is_dup = kept.iter().rev().take(5).any(|k| {
            k.text == label.text && (k.x - label.x).abs() < 0.5 && (k.y - label.y).abs() < 0.01
        });
        if !is_dup {
            kept.push(label);
        }
    }
    *labels = kept;
}

/// Color for road geometry based on road class.
fn road_color(layer: &str, props: &std::collections::HashMap<String, String>) -> Option<Color> {
    if layer != "transportation" {
        return None;
    }
    let class = props.get("class").map(|s| s.as_str()).unwrap_or("");
    match class {
        "motorway" => Some(Color::Rgb(200, 140, 50)), // orange
        "trunk" => Some(Color::Rgb(180, 120, 40)),    // darker orange
        "primary" => Some(Color::Rgb(150, 110, 50)),  // muted orange
        "secondary" => Some(Color::Rgb(100, 90, 60)), // dim
        "tertiary" => Some(Color::Rgb(80, 80, 80)),
        _ => Some(Color::Rgb(55, 55, 55)), // minor roads very dim
    }
}

fn layer_color(layer: &str, props: &std::collections::HashMap<String, String>) -> Option<Color> {
    match layer {
        "water" => Some(Color::Blue),
        "waterway" => Some(Color::Blue),
        "landuse" => Some(Color::DarkGray),
        "landcover" => Some(Color::DarkGray),
        "park" => {
            // Only render terrestrial parks, skip marine protected areas
            let class = props.get("class").map(|s| s.as_str()).unwrap_or("");
            match class {
                "national_park" => Some(Color::Green),
                "nature_reserve" => Some(Color::Rgb(30, 80, 30)),
                // Skip protected_area, natura_2000, réserve_de_la_biosphère,
                // zona_de_especial_*, site_of_special_*, etc. — mostly marine zones
                _ => None,
            }
        }
        "building" => Some(Color::Rgb(60, 60, 60)),
        "boundary" => Some(Color::Rgb(100, 100, 100)),
        _ => None,
    }
}

fn label_color(
    layer: &str,
    props: &std::collections::HashMap<String, String>,
    zoom: u32,
) -> Option<Color> {
    match layer {
        "place" => {
            let class = props.get("class").map(|s| s.as_str()).unwrap_or("");
            match class {
                // z0-z3: countries only
                "country" => Some(Color::White),
                // z4+: states/regions
                "state" if zoom >= 4 => Some(Color::Rgb(180, 180, 180)),
                // z6+: cities
                "city" if zoom >= 6 => Some(Color::Rgb(200, 200, 200)),
                // z8+: towns
                "town" if zoom >= 8 => Some(Color::Rgb(150, 150, 150)),
                // z10+: villages
                "village" if zoom >= 10 => Some(Color::Rgb(120, 120, 120)),
                _ => None,
            }
        }
        "transportation_name" => {
            let class = props.get("class").map(|s| s.as_str()).unwrap_or("");
            match class {
                // z6+: motorways
                "motorway" if zoom >= 6 => Some(Color::Rgb(200, 150, 80)),
                // z8+: trunk/primary
                "trunk" | "primary" if zoom >= 8 => Some(Color::Rgb(160, 130, 70)),
                _ => None,
            }
        }
        // z4+: water names
        "water_name" if zoom >= 4 => Some(Color::Rgb(80, 80, 180)),
        _ => None,
    }
}
