use std::f64::consts::PI;

/// Maximum number of tiles to fetch/render per frame.
/// Beyond this, the map degrades to partial rendering — better than blocking on HTTP fanout.
pub const MAX_VISIBLE_TILES: usize = 16;

pub fn ll2tile(lat: f64, lon: f64, zoom: u32) -> (u32, u32) {
    // Clamp latitude to the Web Mercator valid range; beyond this the
    // projection diverges and y casts produce garbage values that the
    // downstream .min(max_tile) only partially papers over.
    let lat = lat.clamp(-85.05112878, 85.05112878);
    let n = 2.0_f64.powi(zoom as i32);
    let x = ((lon + 180.0) / 360.0 * n).floor() as u32;
    let lat_rad = lat.to_radians();
    let y = ((1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / PI) / 2.0 * n).floor() as u32;
    let max_tile = n as u32 - 1;
    (x.min(max_tile), y.min(max_tile))
}

pub fn tile2ll(x: u32, y: u32, zoom: u32) -> (f64, f64) {
    let n = 2.0_f64.powi(zoom as i32);
    let lon = x as f64 / n * 360.0 - 180.0;
    let lat_rad = (PI * (1.0 - 2.0 * y as f64 / n)).sinh().atan();
    let lat = lat_rad.to_degrees();
    (lat, lon)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileCoord {
    pub z: u32,
    pub x: u32,
    pub y: u32,
}

pub fn visible_tiles(
    min_lat: f64,
    max_lat: f64,
    min_lon: f64,
    max_lon: f64,
    zoom: u32,
) -> Vec<TileCoord> {
    let (x_min, y_max) = ll2tile(min_lat.max(-85.05), min_lon, zoom);
    let (x_max, y_min) = ll2tile(max_lat.min(85.05), max_lon, zoom);
    let mut tiles = Vec::new();
    for x in x_min..=x_max {
        for y in y_min..=y_max {
            tiles.push(TileCoord { z: zoom, x, y });
        }
    }
    tiles
}

pub fn tile_point_to_ll(tile: &TileCoord, px: f64, py: f64, extent: f64) -> (f64, f64) {
    let (top_lat, left_lon) = tile2ll(tile.x, tile.y, tile.z);
    let (bottom_lat, right_lon) = tile2ll(tile.x + 1, tile.y + 1, tile.z);
    let lon = left_lon + (px / extent) * (right_lon - left_lon);
    let lat = top_lat + (py / extent) * (bottom_lat - top_lat);
    (lat, lon)
}
