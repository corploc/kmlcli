use color_eyre::eyre::Result;
use serde::Serialize;
use serde_json;

use crate::model::{Feature, Geometry, KmlDocument};

#[derive(Serialize)]
struct Entry {
    name: String,
    kind: &'static str,
    geometry: Option<&'static str>,
    path: String,
}

fn geometry_type(geom: &Geometry) -> &'static str {
    match geom {
        Geometry::Point(_) => "point",
        Geometry::LineString(_) => "linestring",
        Geometry::Polygon(_) => "polygon",
        Geometry::MultiGeometry(_) => "multi",
    }
}

fn collect_entries(features: &[Feature], path_prefix: &str, entries: &mut Vec<Entry>) {
    for f in features {
        match f {
            Feature::Folder {
                name,
                features: children,
            } => {
                let path = if path_prefix.is_empty() {
                    name.clone()
                } else {
                    format!("{}/{}", path_prefix, name)
                };
                entries.push(Entry {
                    name: name.clone(),
                    kind: "folder",
                    geometry: None,
                    path: path.clone(),
                });
                collect_entries(children, &path, entries);
            }
            Feature::Placemark { name, geometry, .. } => {
                let path = if path_prefix.is_empty() {
                    name.clone()
                } else {
                    format!("{}/{}", path_prefix, name)
                };
                entries.push(Entry {
                    name: name.clone(),
                    kind: "placemark",
                    geometry: geometry.as_ref().map(geometry_type),
                    path,
                });
            }
        }
    }
}

pub fn run(doc: &KmlDocument) -> Result<()> {
    let mut entries = Vec::new();
    collect_entries(&doc.features, "", &mut entries);
    println!("{}", serde_json::to_string_pretty(&entries)?);
    Ok(())
}
