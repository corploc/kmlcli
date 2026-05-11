use ratatui::style::Color;

use super::decode::DecodedFeature;
use super::proto::GeomType;

/// Pre-rendered tile data, computed once at decode time.
#[derive(Clone)]
pub struct RenderedTile {
    pub segments: Vec<TileSegment>,
    pub labels: Vec<TileLabel>,
}

#[derive(Clone, Copy)]
pub struct TileSegment {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub color: Color,
}

#[derive(Clone)]
pub struct TileLabel {
    pub x: f64,
    pub y: f64,
    pub text: String,
    pub color: Color,
    pub min_zoom: u32,
}

/// Pre-render a decoded tile into segments and label candidates.
/// Called once in the worker thread — no per-frame cost.
/// Coordinates are in (lon, mercator_y(lat)) space — viewport-independent.
pub fn prerender_tile(features: &[DecodedFeature]) -> RenderedTile {
    let mut segments = Vec::new();
    let mut labels = Vec::new();

    for feature in features {
        // Segments
        if let Some(color) = road_color(&feature.layer, &feature.properties)
            .or_else(|| layer_color(&feature.layer, &feature.properties))
        {
            if matches!(feature.geom_type, GeomType::LineString | GeomType::Polygon) {
                for ring in &feature.rings {
                    // Douglas-Peucker-lite: skip segments shorter than ~0.0001°
                    // (invisible at any reasonable terminal resolution)
                    let min_len_sq = 1e-8;
                    for window in ring.windows(2) {
                        let (x1, y1) = project(window[0].0, window[0].1);
                        let (x2, y2) = project(window[1].0, window[1].1);
                        let dx = x2 - x1;
                        let dy = y2 - y1;
                        if dx * dx + dy * dy < min_len_sq {
                            continue;
                        }
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
        }

        // Labels
        let name = match feature.properties.get("name") {
            Some(n) if !n.is_empty() => n,
            _ => continue,
        };
        if let Some((color, min_zoom)) = label_style(&feature.layer, &feature.properties) {
            if let Some(&(lon, lat)) = feature.rings.first().and_then(|r| r.first()) {
                let (x, y) = project(lon, lat);
                labels.push(TileLabel {
                    x,
                    y,
                    text: name.clone(),
                    color,
                    min_zoom,
                });
            }
        }
    }

    RenderedTile { segments, labels }
}

/// Deduplicate labels that have the same text and are close together.
/// O(n log n) — sort then single-pass retain.
pub fn dedup_labels(labels: &mut Vec<TileLabel>) {
    labels.sort_by(|a, b| a.text.cmp(&b.text));

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

/// Mercator projection: (lon, lat) → (x, y) in canvas space.
/// Identical to Viewport::project_for_canvas but without needing a viewport ref.
fn project(lon: f64, lat: f64) -> (f64, f64) {
    let lat_clamped = lat.clamp(-85.05, 85.05);
    let lat_rad = lat_clamped.to_radians();
    (
        lon,
        (std::f64::consts::FRAC_PI_4 + lat_rad / 2.0).tan().ln(),
    )
}

fn road_color(layer: &str, props: &std::collections::HashMap<String, String>) -> Option<Color> {
    if layer != "transportation" {
        return None;
    }
    let class = props.get("class").map(|s| s.as_str()).unwrap_or("");
    match class {
        "motorway" => Some(Color::Rgb(200, 140, 50)),
        "trunk" => Some(Color::Rgb(180, 120, 40)),
        "primary" => Some(Color::Rgb(150, 110, 50)),
        "secondary" => Some(Color::Rgb(100, 90, 60)),
        "tertiary" => Some(Color::Rgb(80, 80, 80)),
        _ => Some(Color::Rgb(55, 55, 55)),
    }
}

fn layer_color(layer: &str, props: &std::collections::HashMap<String, String>) -> Option<Color> {
    match layer {
        "water" => Some(Color::Blue),
        "waterway" => Some(Color::Blue),
        "landuse" => Some(Color::DarkGray),
        "landcover" => Some(Color::DarkGray),
        "park" => {
            let class = props.get("class").map(|s| s.as_str()).unwrap_or("");
            match class {
                "national_park" => Some(Color::Green),
                "nature_reserve" => Some(Color::Rgb(30, 80, 30)),
                _ => None,
            }
        }
        "building" => Some(Color::Rgb(60, 60, 60)),
        "boundary" => Some(Color::Rgb(100, 100, 100)),
        _ => None,
    }
}

/// Returns (color, min_zoom) for label display.
fn label_style(
    layer: &str,
    props: &std::collections::HashMap<String, String>,
) -> Option<(Color, u32)> {
    match layer {
        "place" => {
            let class = props.get("class").map(|s| s.as_str()).unwrap_or("");
            match class {
                "country" => Some((Color::White, 0)),
                "state" => Some((Color::Rgb(180, 180, 180), 4)),
                "city" => Some((Color::Rgb(200, 200, 200), 6)),
                "town" => Some((Color::Rgb(150, 150, 150), 8)),
                "village" => Some((Color::Rgb(120, 120, 120), 10)),
                _ => None,
            }
        }
        "transportation_name" => {
            let class = props.get("class").map(|s| s.as_str()).unwrap_or("");
            match class {
                "motorway" => Some((Color::Rgb(200, 150, 80), 6)),
                "trunk" | "primary" => Some((Color::Rgb(160, 130, 70), 8)),
                _ => None,
            }
        }
        "water_name" => Some((Color::Rgb(80, 80, 180), 4)),
        _ => None,
    }
}
