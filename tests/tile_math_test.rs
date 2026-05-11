use kmlcli::tiles::math::{ll2tile, tile2ll, visible_tiles, TileCoord};

#[test]
fn test_ll2tile_zoom_0() {
    let (x, y) = ll2tile(0.0, 0.0, 0);
    assert_eq!(x, 0);
    assert_eq!(y, 0);
}

#[test]
fn test_ll2tile_zoom_2_paris() {
    let (x, y) = ll2tile(48.86, 2.35, 2);
    assert_eq!(x, 2);
    assert_eq!(y, 1);
}

#[test]
fn test_tile2ll_roundtrip() {
    let zoom = 10;
    let (x, y) = ll2tile(48.86, 2.35, zoom);
    let (lat, lon) = tile2ll(x, y, zoom);
    assert!((lat - 48.86).abs() < 1.0, "lat={lat}");
    assert!((lon - 2.35).abs() < 1.0, "lon={lon}");
}

#[test]
fn test_visible_tiles_returns_nonempty() {
    let tiles = visible_tiles(48.0, 49.0, 1.0, 3.0, 10);
    assert!(!tiles.is_empty());
    assert!(tiles.len() > 1);
    assert!(tiles.len() < 100);
}

#[test]
fn test_visible_tiles_zoom_0_is_one_tile() {
    let tiles = visible_tiles(-85.0, 85.0, -180.0, 180.0, 0);
    assert_eq!(tiles.len(), 1);
    assert_eq!(tiles[0], TileCoord { z: 0, x: 0, y: 0 });
}
