use super::math::TileCoord;
use super::proto::{GeomType, Tile};

#[derive(Clone)]
pub struct DecodedFeature {
    pub layer: String,
    pub geom_type: GeomType,
    pub rings: Vec<Vec<(f64, f64)>>, // Vec of rings, each ring is Vec<(lon, lat)>
}

pub fn decode_tile(tile: &Tile, coord: &TileCoord) -> Vec<DecodedFeature> {
    let mut features = Vec::new();
    for layer in &tile.layers {
        let extent = layer.extent.unwrap_or(4096) as f64;
        let layer_name = &layer.name;

        for feature in &layer.features {
            let geom_type = feature
                .r#type
                .map(GeomType::from_i32)
                .unwrap_or(GeomType::Unknown);
            let rings = decode_geometry(&feature.geometry, coord, extent);
            if rings.is_empty() {
                continue;
            }
            features.push(DecodedFeature {
                layer: layer_name.clone(),
                geom_type,
                rings,
            });
        }
    }
    features
}

fn decode_geometry(geometry: &[u32], coord: &TileCoord, extent: f64) -> Vec<Vec<(f64, f64)>> {
    let mut rings: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut current_ring: Vec<(f64, f64)> = Vec::new();
    let mut cx: i32 = 0;
    let mut cy: i32 = 0;
    let mut i = 0;

    while i < geometry.len() {
        let cmd_int = geometry[i];
        let cmd_id = cmd_int & 0x7;
        let count = (cmd_int >> 3) as usize;
        i += 1;

        match cmd_id {
            1 => {
                // MoveTo
                if !current_ring.is_empty() {
                    rings.push(std::mem::take(&mut current_ring));
                }
                for _ in 0..count {
                    if i + 1 >= geometry.len() {
                        break;
                    }
                    cx += zigzag_decode(geometry[i]);
                    cy += zigzag_decode(geometry[i + 1]);
                    i += 2;
                    let (lat, lon) =
                        super::math::tile_point_to_ll(coord, cx as f64, cy as f64, extent);
                    current_ring.push((lon, lat));
                }
            }
            2 => {
                // LineTo
                for _ in 0..count {
                    if i + 1 >= geometry.len() {
                        break;
                    }
                    cx += zigzag_decode(geometry[i]);
                    cy += zigzag_decode(geometry[i + 1]);
                    i += 2;
                    let (lat, lon) =
                        super::math::tile_point_to_ll(coord, cx as f64, cy as f64, extent);
                    current_ring.push((lon, lat));
                }
            }
            7 => {
                // ClosePath
                if let Some(first) = current_ring.first() {
                    current_ring.push(*first);
                }
            }
            _ => {}
        }
    }

    if !current_ring.is_empty() {
        rings.push(current_ring);
    }
    rings
}

fn zigzag_decode(val: u32) -> i32 {
    ((val >> 1) as i32) ^ (-((val & 1) as i32))
}

/// Public wrapper for testing.
pub fn zigzag_decode_pub(val: u32) -> i32 {
    zigzag_decode(val)
}
