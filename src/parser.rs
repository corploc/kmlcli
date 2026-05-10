use std::collections::HashMap;
use std::path::Path;

use color_eyre::eyre::{eyre, Result};
use kml::reader::KmlReader;
use kml::types::{Folder as KmlFolder, Geometry as KmlGeometry, Kml, Placemark as KmlPlacemark};

use crate::model::{Coord, Feature, Geometry, KmlDocument, Style};

pub fn parse_file(path: &Path) -> Result<KmlDocument> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let kml_data: Kml<f64> = match ext.as_str() {
        "kmz" => {
            let mut reader = KmlReader::<_, f64>::from_kmz_path(path)
                .map_err(|e| eyre!("Failed to open KMZ: {e}"))?;
            reader
                .read()
                .map_err(|e| eyre!("Failed to parse KMZ: {e}"))?
        }
        _ => {
            let mut reader = KmlReader::<_, f64>::from_path(path)
                .map_err(|e| eyre!("Failed to read KML: {e}"))?;
            reader
                .read()
                .map_err(|e| eyre!("Failed to parse KML: {e}"))?
        }
    };

    let mut doc = KmlDocument {
        name: None,
        features: Vec::new(),
        styles: HashMap::new(),
    };

    convert_kml(&kml_data, &mut doc);
    Ok(doc)
}

fn convert_kml(kml: &Kml<f64>, doc: &mut KmlDocument) {
    match kml {
        Kml::KmlDocument(kml_doc) => {
            for el in &kml_doc.elements {
                convert_kml(el, doc);
            }
        }
        Kml::Document { elements, .. } => {
            // Extract name from elements (stored as Element nodes)
            for el in elements {
                if let Kml::Element(e) = el {
                    if e.name == "name" {
                        doc.name = e.content.clone();
                    }
                }
            }
            // Process all other elements
            for el in elements {
                match el {
                    Kml::Element(_) => {} // already handled above
                    _ => process_document_child(el, doc),
                }
            }
        }
        Kml::Folder(folder) => {
            let feature = convert_folder(folder, doc);
            doc.features.push(feature);
        }
        Kml::Placemark(pm) => {
            let feature = convert_placemark(pm);
            doc.features.push(feature);
        }
        Kml::Style(style) => {
            if let Some(id) = &style.id {
                let our_style = Style {
                    id: id.clone(),
                    line_color: style.line.as_ref().map(|l| l.color.clone()),
                    line_width: style.line.as_ref().map(|l| l.width),
                    poly_color: style.poly.as_ref().map(|p| p.color.clone()),
                    icon_href: style
                        .icon
                        .as_ref()
                        .map(|i| i.icon.href.clone())
                        .filter(|s| !s.is_empty()),
                };
                doc.styles.insert(id.clone(), our_style);
            }
        }
        _ => {}
    }
}

fn process_document_child(kml: &Kml<f64>, doc: &mut KmlDocument) {
    match kml {
        Kml::Folder(folder) => {
            let feature = convert_folder(folder, doc);
            doc.features.push(feature);
        }
        Kml::Placemark(pm) => {
            let feature = convert_placemark(pm);
            doc.features.push(feature);
        }
        Kml::Style(style) => {
            if let Some(id) = &style.id {
                let our_style = Style {
                    id: id.clone(),
                    line_color: style.line.as_ref().map(|l| l.color.clone()),
                    line_width: style.line.as_ref().map(|l| l.width),
                    poly_color: style.poly.as_ref().map(|p| p.color.clone()),
                    icon_href: style
                        .icon
                        .as_ref()
                        .map(|i| i.icon.href.clone())
                        .filter(|s| !s.is_empty()),
                };
                doc.styles.insert(id.clone(), our_style);
            }
        }
        _ => {}
    }
}

fn convert_folder(folder: &KmlFolder<f64>, doc: &mut KmlDocument) -> Feature {
    let name = folder.name.clone().unwrap_or_default();
    let mut features = Vec::new();

    for el in &folder.elements {
        match el {
            Kml::Placemark(pm) => {
                features.push(convert_placemark(pm));
            }
            Kml::Folder(sub_folder) => {
                features.push(convert_folder(sub_folder, doc));
            }
            Kml::Style(style) => {
                if let Some(id) = &style.id {
                    let our_style = Style {
                        id: id.clone(),
                        line_color: style.line.as_ref().map(|l| l.color.clone()),
                        line_width: style.line.as_ref().map(|l| l.width),
                        poly_color: style.poly.as_ref().map(|p| p.color.clone()),
                        icon_href: style
                            .icon
                            .as_ref()
                            .map(|i| i.icon.href.clone())
                            .filter(|s| !s.is_empty()),
                    };
                    doc.styles.insert(id.clone(), our_style);
                }
            }
            _ => {}
        }
    }

    Feature::Folder { name, features }
}

fn convert_placemark(pm: &KmlPlacemark<f64>) -> Feature {
    let name = pm.name.clone().unwrap_or_default();
    let geometry = pm.geometry.as_ref().map(convert_geometry);
    let style_id = pm
        .style_url
        .as_ref()
        .map(|s| s.trim_start_matches('#').to_string());
    let description = pm.description.clone();

    Feature::Placemark {
        name,
        geometry,
        style_id,
        description,
    }
}

fn convert_geometry(geom: &KmlGeometry<f64>) -> Geometry {
    match geom {
        KmlGeometry::Point(p) => Geometry::Point(Coord {
            lon: p.coord.x,
            lat: p.coord.y,
            alt: p.coord.z,
        }),
        KmlGeometry::LineString(ls) => Geometry::LineString(
            ls.coords
                .iter()
                .map(|c| Coord {
                    lon: c.x,
                    lat: c.y,
                    alt: c.z,
                })
                .collect(),
        ),
        KmlGeometry::LinearRing(lr) => Geometry::LineString(
            lr.coords
                .iter()
                .map(|c| Coord {
                    lon: c.x,
                    lat: c.y,
                    alt: c.z,
                })
                .collect(),
        ),
        KmlGeometry::Polygon(poly) => {
            let mut rings: Vec<Vec<Coord>> = Vec::new();
            // outer ring first
            let outer: Vec<Coord> = poly
                .outer
                .coords
                .iter()
                .map(|c| Coord {
                    lon: c.x,
                    lat: c.y,
                    alt: c.z,
                })
                .collect();
            rings.push(outer);
            // inner rings
            for inner in &poly.inner {
                rings.push(
                    inner
                        .coords
                        .iter()
                        .map(|c| Coord {
                            lon: c.x,
                            lat: c.y,
                            alt: c.z,
                        })
                        .collect(),
                );
            }
            Geometry::Polygon(rings)
        }
        KmlGeometry::MultiGeometry(mg) => {
            Geometry::MultiGeometry(mg.geometries.iter().map(convert_geometry).collect())
        }
        // Element (Model placeholder) and any future non-exhaustive variants
        _ => Geometry::MultiGeometry(Vec::new()),
    }
}
