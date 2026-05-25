use kmlcli::model::{BoundingBox, Coord};
use kmlcli::projection::Viewport;

#[test]
fn test_project_center_of_viewport_maps_to_center_of_canvas() {
    let bbox = BoundingBox {
        min_lon: -2.0,
        max_lon: 0.0,
        min_lat: 47.0,
        max_lat: 49.0,
    };
    let vp = Viewport::from_bbox(&bbox);
    let (x, y) = vp.project(&Coord {
        lon: -1.0,
        lat: 48.0,
        alt: None,
    });
    assert!((x - 0.5).abs() < 0.05, "x={x} expected ~0.5");
    assert!((y - 0.5).abs() < 0.05, "y={y} expected ~0.5");
}

#[test]
fn test_project_corners_map_to_bounds() {
    let bbox = BoundingBox {
        min_lon: 0.0,
        max_lon: 10.0,
        min_lat: 0.0,
        max_lat: 10.0,
    };
    let vp = Viewport::from_bbox(&bbox);
    let (x, y) = vp.project(&Coord {
        lon: 0.0,
        lat: 0.0,
        alt: None,
    });
    assert!(x < 0.05, "bottom-left x={x}");
    assert!(y < 0.05, "bottom-left y={y}");
    let (x, y) = vp.project(&Coord {
        lon: 10.0,
        lat: 10.0,
        alt: None,
    });
    assert!(x > 0.95, "top-right x={x}");
    assert!(y > 0.95, "top-right y={y}");
}

#[test]
fn test_zoom_in_narrows_bounds() {
    let bbox = BoundingBox {
        min_lon: 0.0,
        max_lon: 10.0,
        min_lat: 0.0,
        max_lat: 10.0,
    };
    let mut vp = Viewport::from_bbox(&bbox);
    let original_width = vp.lon_span();
    vp.zoom_in();
    assert!(vp.lon_span() < original_width);
}

#[test]
fn pan_up_clamps_to_mercator_max() {
    let bbox = BoundingBox {
        min_lon: 0.0,
        max_lon: 1.0,
        min_lat: 84.0,
        max_lat: 85.0,
    };
    let mut vp = Viewport::from_bbox(&bbox);
    for _ in 0..1000 {
        vp.pan_up();
    }
    assert!(
        vp.center_lat <= 85.05,
        "center_lat = {} exceeded 85.05",
        vp.center_lat
    );
}

#[test]
fn pan_down_clamps_to_mercator_min() {
    let bbox = BoundingBox {
        min_lon: 0.0,
        max_lon: 1.0,
        min_lat: -85.0,
        max_lat: -84.0,
    };
    let mut vp = Viewport::from_bbox(&bbox);
    for _ in 0..1000 {
        vp.pan_down();
    }
    assert!(
        vp.center_lat >= -85.05,
        "center_lat = {} below -85.05",
        vp.center_lat
    );
}

#[test]
fn test_pan_shifts_center() {
    let bbox = BoundingBox {
        min_lon: 0.0,
        max_lon: 10.0,
        min_lat: 0.0,
        max_lat: 10.0,
    };
    let mut vp = Viewport::from_bbox(&bbox);
    let original_center = vp.center_lon;
    vp.pan_right();
    assert!(vp.center_lon > original_center);
}
