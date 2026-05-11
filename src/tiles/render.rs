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
