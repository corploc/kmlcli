use crate::model::{BoundingBox, Coord};

const ZOOM_FACTOR: f64 = 0.8;
const PAN_FACTOR: f64 = 0.1;

#[derive(Debug, Clone)]
pub struct Viewport {
    pub center_lon: f64,
    pub center_lat: f64,
    half_lon: f64,
    half_lat: f64,
}

impl Viewport {
    pub fn from_bbox(bbox: &BoundingBox) -> Self {
        let center_lon = (bbox.min_lon + bbox.max_lon) / 2.0;
        let center_lat = (bbox.min_lat + bbox.max_lat) / 2.0;
        let half_lon = (bbox.max_lon - bbox.min_lon) / 2.0 * 1.1;
        let half_lat = (bbox.max_lat - bbox.min_lat) / 2.0 * 1.1;
        Self {
            center_lon,
            center_lat,
            half_lon,
            half_lat,
        }
    }

    pub fn project(&self, coord: &Coord) -> (f64, f64) {
        let x = (coord.lon - (self.center_lon - self.half_lon)) / (2.0 * self.half_lon);
        let y = (mercator_y(coord.lat) - mercator_y(self.center_lat - self.half_lat))
            / (mercator_y(self.center_lat + self.half_lat)
                - mercator_y(self.center_lat - self.half_lat));
        (x, y)
    }

    pub fn x_bounds(&self) -> [f64; 2] {
        [
            self.center_lon - self.half_lon,
            self.center_lon + self.half_lon,
        ]
    }

    pub fn y_bounds(&self) -> [f64; 2] {
        let min_lat = (self.center_lat - self.half_lat).max(-85.05);
        let max_lat = (self.center_lat + self.half_lat).min(85.05);
        [mercator_y(min_lat), mercator_y(max_lat)]
    }

    pub fn project_for_canvas(&self, coord: &Coord) -> (f64, f64) {
        (coord.lon, mercator_y(coord.lat.clamp(-85.05, 85.05)))
    }

    pub fn lon_span(&self) -> f64 {
        self.half_lon * 2.0
    }
    pub fn zoom_in(&mut self) {
        self.half_lon *= ZOOM_FACTOR;
        self.half_lat *= ZOOM_FACTOR;
    }
    pub fn zoom_out(&mut self) {
        self.half_lon /= ZOOM_FACTOR;
        self.half_lat /= ZOOM_FACTOR;
    }
    pub fn pan_left(&mut self) {
        self.center_lon -= self.half_lon * PAN_FACTOR;
    }
    pub fn pan_right(&mut self) {
        self.center_lon += self.half_lon * PAN_FACTOR;
    }
    pub fn pan_up(&mut self) {
        self.center_lat = (self.center_lat + self.half_lat * PAN_FACTOR).min(85.05);
    }
    pub fn pan_down(&mut self) {
        self.center_lat = (self.center_lat - self.half_lat * PAN_FACTOR).max(-85.05);
    }
    pub fn center_on(&mut self, coord: &Coord) {
        self.center_lon = coord.lon;
        self.center_lat = coord.lat;
    }
    pub fn intersects(&self, bbox: &BoundingBox) -> bool {
        bbox.max_lon >= self.center_lon - self.half_lon
            && bbox.min_lon <= self.center_lon + self.half_lon
            && bbox.max_lat >= self.center_lat - self.half_lat
            && bbox.min_lat <= self.center_lat + self.half_lat
    }

    /// Compute the OSM zoom level (0-18) matching the current viewport span.
    pub fn zoom_level(&self) -> u32 {
        let z = (360.0 / (self.half_lon * 2.0)).log2();
        z.round().clamp(0.0, 18.0) as u32
    }

    /// Returns lat bounds (not mercator-projected).
    pub fn lat_bounds(&self) -> [f64; 2] {
        [
            self.center_lat - self.half_lat,
            self.center_lat + self.half_lat,
        ]
    }
}

fn mercator_y(lat: f64) -> f64 {
    let lat_rad = lat.to_radians();
    (std::f64::consts::FRAC_PI_4 + lat_rad / 2.0).tan().ln()
}
