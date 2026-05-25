use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Coord {
    pub lon: f64,
    pub lat: f64,
    pub alt: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub enum Geometry {
    Point(Coord),
    LineString(Vec<Coord>),
    Polygon(Vec<Vec<Coord>>),
    MultiGeometry(Vec<Geometry>),
}

#[derive(Debug, Clone, Serialize)]
pub enum Feature {
    Folder {
        name: String,
        features: Vec<Feature>,
    },
    Placemark {
        name: String,
        geometry: Option<Geometry>,
        style_id: Option<String>,
        description: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct Style {
    pub id: String,
    pub line_color: Option<String>,
    pub line_width: Option<f64>,
    pub poly_color: Option<String>,
    pub icon_href: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KmlDocument {
    pub name: Option<String>,
    pub features: Vec<Feature>,
    pub styles: HashMap<String, Style>,
    /// Maps StyleMap id → target Style id (the "normal" pair).
    /// KML StyleMap is an indirection layer Google Maps exports use heavily.
    pub style_maps: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min_lon: f64,
    pub max_lon: f64,
    pub min_lat: f64,
    pub max_lat: f64,
}

impl KmlDocument {
    pub fn all_coords(&self) -> Vec<&Coord> {
        let mut coords = Vec::new();
        for feature in &self.features {
            collect_coords(feature, &mut coords);
        }
        coords
    }

    pub fn bounding_box(&self) -> Option<BoundingBox> {
        let coords = self.all_coords();
        if coords.is_empty() {
            return None;
        }
        let mut bbox = BoundingBox {
            min_lon: f64::MAX,
            max_lon: f64::MIN,
            min_lat: f64::MAX,
            max_lat: f64::MIN,
        };
        for c in &coords {
            bbox.min_lon = bbox.min_lon.min(c.lon);
            bbox.max_lon = bbox.max_lon.max(c.lon);
            bbox.min_lat = bbox.min_lat.min(c.lat);
            bbox.max_lat = bbox.max_lat.max(c.lat);
        }
        if (bbox.max_lon - bbox.min_lon).abs() < 1e-9 {
            bbox.min_lon -= 0.001;
            bbox.max_lon += 0.001;
        }
        if (bbox.max_lat - bbox.min_lat).abs() < 1e-9 {
            bbox.min_lat -= 0.001;
            bbox.max_lat += 0.001;
        }
        Some(bbox)
    }

    /// Resolve a style id, following one level of StyleMap indirection.
    pub fn resolve_style(&self, id: &str) -> Option<&Style> {
        if let Some(s) = self.styles.get(id) {
            return Some(s);
        }
        let target = self.style_maps.get(id)?;
        self.styles.get(target)
    }

    pub fn flatten(&self) -> Vec<(Vec<usize>, &Feature)> {
        let mut result = Vec::new();
        for (i, feature) in self.features.iter().enumerate() {
            flatten_recursive(feature, vec![i], &mut result);
        }
        result
    }
}

fn collect_coords<'a>(feature: &'a Feature, coords: &mut Vec<&'a Coord>) {
    match feature {
        Feature::Folder { features, .. } => {
            for f in features {
                collect_coords(f, coords);
            }
        }
        Feature::Placemark { geometry, .. } => {
            if let Some(geom) = geometry {
                collect_geom_coords(geom, coords);
            }
        }
    }
}

fn collect_geom_coords<'a>(geom: &'a Geometry, coords: &mut Vec<&'a Coord>) {
    match geom {
        Geometry::Point(c) => coords.push(c),
        Geometry::LineString(cs) => coords.extend(cs),
        Geometry::Polygon(rings) => {
            for ring in rings {
                coords.extend(ring);
            }
        }
        Geometry::MultiGeometry(geoms) => {
            for g in geoms {
                collect_geom_coords(g, coords);
            }
        }
    }
}

fn flatten_recursive<'a>(
    feature: &'a Feature,
    path: Vec<usize>,
    result: &mut Vec<(Vec<usize>, &'a Feature)>,
) {
    result.push((path.clone(), feature));
    if let Feature::Folder { features, .. } = feature {
        for (i, child) in features.iter().enumerate() {
            let mut child_path = path.clone();
            child_path.push(i);
            flatten_recursive(child, child_path, result);
        }
    }
}
