use crate::model::{Feature, Geometry, KmlDocument};

fn geometry_icon(geom: &Geometry) -> &'static str {
    match geom {
        Geometry::Point(_) => "●",
        Geometry::LineString(_) => "─",
        Geometry::Polygon(_) => "◻",
        Geometry::MultiGeometry(_) => "◈",
    }
}

fn print_features(features: &[Feature], prefix: &str) {
    let count = features.len();
    for (i, feature) in features.iter().enumerate() {
        let is_last = i == count - 1;
        let connector = if is_last { "└──" } else { "├──" };
        let child_prefix = if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };

        match feature {
            Feature::Folder {
                name,
                features: children,
            } => {
                println!("{}{} 📁 {}", prefix, connector, name);
                print_features(children, &child_prefix);
            }
            Feature::Placemark { name, geometry, .. } => {
                let icon = geometry.as_ref().map(geometry_icon).unwrap_or("·");
                println!("{}{} {} {}", prefix, connector, icon, name);
            }
        }
    }
}

pub fn run(doc: &KmlDocument) {
    let root_name = doc.name.as_deref().unwrap_or("(unnamed)");
    println!("📄 {}", root_name);
    print_features(&doc.features, "");
}
