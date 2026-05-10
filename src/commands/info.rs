use color_eyre::eyre::Result;
use serde::Serialize;
use serde_json;

use crate::model::{Feature, KmlDocument};

#[derive(Serialize)]
struct BoundingBoxJson {
    min_lon: f64,
    max_lon: f64,
    min_lat: f64,
    max_lat: f64,
}

#[derive(Serialize)]
struct InfoOutput {
    name: Option<String>,
    feature_count: usize,
    placemark_count: usize,
    folder_count: usize,
    style_count: usize,
    bounding_box: Option<BoundingBoxJson>,
}

fn count_features(features: &[Feature]) -> (usize, usize) {
    let mut placemarks = 0;
    let mut folders = 0;
    for f in features {
        match f {
            Feature::Folder {
                features: children, ..
            } => {
                folders += 1;
                let (p, fo) = count_features(children);
                placemarks += p;
                folders += fo;
            }
            Feature::Placemark { .. } => {
                placemarks += 1;
            }
        }
    }
    (placemarks, folders)
}

fn count_all_features(features: &[Feature]) -> usize {
    let mut total = 0;
    for f in features {
        total += 1;
        if let Feature::Folder {
            features: children, ..
        } = f
        {
            total += count_all_features(children);
        }
    }
    total
}

pub fn run(doc: &KmlDocument) -> Result<()> {
    let (placemark_count, folder_count) = count_features(&doc.features);
    let feature_count = count_all_features(&doc.features);
    let bounding_box = doc.bounding_box().map(|bb| BoundingBoxJson {
        min_lon: bb.min_lon,
        max_lon: bb.max_lon,
        min_lat: bb.min_lat,
        max_lat: bb.max_lat,
    });

    let output = InfoOutput {
        name: doc.name.clone(),
        feature_count,
        placemark_count,
        folder_count,
        style_count: doc.styles.len(),
        bounding_box,
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
