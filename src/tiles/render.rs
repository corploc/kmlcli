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
        let color = match layer_color(&feature.layer) {
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

/// Extract labels from decoded tile features (place names, road names, water names).
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

fn layer_color(layer: &str) -> Option<Color> {
    match layer {
        "water" => Some(Color::Blue),
        "waterway" => Some(Color::Blue),
        "landuse" => Some(Color::DarkGray),
        "landcover" => Some(Color::DarkGray),
        "park" => Some(Color::Green),
        "transportation" => Some(Color::Rgb(80, 80, 80)),
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
                "village" | "hamlet" | "suburb" | "quarter" | "neighbourhood" => {
                    Some(Color::Rgb(120, 120, 120))
                }
                _ => Some(Color::Rgb(100, 100, 100)),
            }
        }
        "transportation_name" => Some(Color::Rgb(90, 90, 90)),
        "water_name" => Some(Color::Rgb(80, 80, 180)),
        "mountain_peak" => Some(Color::Rgb(160, 140, 100)),
        "poi" => Some(Color::Rgb(100, 100, 100)),
        _ => None,
    }
}
