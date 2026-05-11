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
/// Only shows major road names, all place names, and water names.
pub fn render_tile_labels(features: &[DecodedFeature], viewport: &Viewport) -> Vec<TileLabel> {
    let mut labels = Vec::new();

    for feature in features {
        let name = match feature.properties.get("name") {
            Some(n) if !n.is_empty() => n,
            _ => continue,
        };

        let color = match label_color(&feature.layer, &feature.properties) {
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
/// Call this after collecting labels from all tiles.
pub fn dedup_labels(labels: &mut Vec<TileLabel>) {
    // Sort by text for grouping
    labels.sort_by(|a, b| a.text.cmp(&b.text));

    let mut i = 0;
    while i < labels.len() {
        let mut j = i + 1;
        while j < labels.len() && labels[j].text == labels[i].text {
            // Same text — check if close (within ~0.01 canvas units)
            let dx = (labels[j].x - labels[i].x).abs();
            let dy = (labels[j].y - labels[i].y).abs();
            if dx < 0.5 && dy < 0.01 {
                labels.remove(j);
            } else {
                j += 1;
            }
        }
        i += 1;
    }
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

fn label_color(layer: &str, props: &std::collections::HashMap<String, String>) -> Option<Color> {
    match layer {
        "place" => {
            let class = props.get("class").map(|s| s.as_str()).unwrap_or("");
            match class {
                "country" => Some(Color::White),
                "state" => Some(Color::Rgb(180, 180, 180)),
                "city" => Some(Color::Rgb(200, 200, 200)),
                "town" => Some(Color::Rgb(150, 150, 150)),
                "village" => Some(Color::Rgb(120, 120, 120)),
                // Skip hamlet, suburb, quarter, neighbourhood — too cluttered
                _ => None,
            }
        }
        "transportation_name" => {
            // Only label major roads
            let class = props.get("class").map(|s| s.as_str()).unwrap_or("");
            match class {
                "motorway" => Some(Color::Rgb(200, 150, 80)),
                "trunk" | "primary" => Some(Color::Rgb(160, 130, 70)),
                // Skip secondary, tertiary, residential, service, etc.
                _ => None,
            }
        }
        "water_name" => Some(Color::Rgb(80, 80, 180)),
        _ => None,
    }
}
