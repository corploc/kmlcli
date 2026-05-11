use ratatui::{
    style::{Color, Style},
    symbols::Marker,
    text::Span,
    widgets::canvas::{Canvas, Line as CanvasLine, Points},
};

use crate::{
    model::{Geometry, KmlDocument},
    projection::Viewport,
    tiles::{
        fetch::TileCache,
        math,
        render::{dedup_labels, render_tile_features, render_tile_labels},
    },
};

pub struct MapView<'a> {
    pub doc: &'a KmlDocument,
    pub viewport: &'a Viewport,
    pub selected_path: Option<&'a [usize]>,
    pub focused: bool,
    pub tile_cache: &'a TileCache,
}

impl<'a> MapView<'a> {
    pub fn new(
        doc: &'a KmlDocument,
        viewport: &'a Viewport,
        selected_path: Option<&'a [usize]>,
        focused: bool,
        tile_cache: &'a TileCache,
    ) -> Self {
        Self {
            doc,
            viewport,
            selected_path,
            focused,
            tile_cache,
        }
    }

    pub fn widget(self) -> impl ratatui::widgets::Widget + 'a {
        let x_bounds = self.viewport.x_bounds();
        let y_bounds = self.viewport.y_bounds();

        // Collect tile background segments + labels
        let zoom = self.viewport.zoom_level().min(16);
        let lat_bounds = self.viewport.lat_bounds();
        let visible =
            math::visible_tiles(lat_bounds[0], lat_bounds[1], x_bounds[0], x_bounds[1], zoom);
        let visible: Vec<_> = visible.into_iter().take(16).collect();

        let mut tile_segments = Vec::new();
        let mut tile_labels = Vec::new();
        for tc in &visible {
            if let Some(features) = self.tile_cache.get_cached(tc) {
                tile_segments.extend(render_tile_features(&features, self.viewport));
                tile_labels.extend(render_tile_labels(&features, self.viewport));
            }
        }
        dedup_labels(&mut tile_labels);

        // Collect KML foreground segments + labels
        let selected_path = self.selected_path.map(|p| p.to_vec());
        let kml_segments = collect_segments(self.doc, self.viewport, &selected_path);
        let kml_labels = collect_labels(self.doc, self.viewport, &selected_path);

        Canvas::default()
            .x_bounds(x_bounds)
            .y_bounds(y_bounds)
            .marker(Marker::Braille)
            .paint(move |ctx| {
                // Background: tile geometry
                for seg in &tile_segments {
                    if let Some((cx1, cy1, cx2, cy2)) =
                        clip_line(seg.x1, seg.y1, seg.x2, seg.y2, &x_bounds, &y_bounds)
                    {
                        ctx.draw(&CanvasLine {
                            x1: cx1,
                            y1: cy1,
                            x2: cx2,
                            y2: cy2,
                            color: seg.color,
                        });
                    }
                }

                // Foreground: KML geometry
                for seg in &kml_segments {
                    match seg {
                        DrawCmd::Line {
                            x1,
                            y1,
                            x2,
                            y2,
                            color,
                        } => {
                            if let Some((cx1, cy1, cx2, cy2)) =
                                clip_line(*x1, *y1, *x2, *y2, &x_bounds, &y_bounds)
                            {
                                ctx.draw(&CanvasLine {
                                    x1: cx1,
                                    y1: cy1,
                                    x2: cx2,
                                    y2: cy2,
                                    color: *color,
                                });
                            }
                        }
                        DrawCmd::Point { x, y, color } => {
                            if *x >= x_bounds[0]
                                && *x <= x_bounds[1]
                                && *y >= y_bounds[0]
                                && *y <= y_bounds[1]
                            {
                                let coords = [(*x, *y)];
                                ctx.draw(&Points {
                                    coords: &coords,
                                    color: *color,
                                });
                            }
                        }
                    }
                }

                // Tile place labels (countries, cities, roads, etc.)
                for label in &tile_labels {
                    if label.x >= x_bounds[0]
                        && label.x <= x_bounds[1]
                        && label.y >= y_bounds[0]
                        && label.y <= y_bounds[1]
                    {
                        ctx.print(
                            label.x,
                            label.y,
                            ratatui::text::Line::from(Span::styled(
                                label.text.clone(),
                                Style::default().fg(label.color),
                            )),
                        );
                    }
                }

                // KML element labels
                for label in &kml_labels {
                    if label.x >= x_bounds[0]
                        && label.x <= x_bounds[1]
                        && label.y >= y_bounds[0]
                        && label.y <= y_bounds[1]
                    {
                        ctx.print(
                            label.x,
                            label.y,
                            ratatui::text::Line::from(Span::styled(
                                label.text.clone(),
                                Style::default().fg(label.color),
                            )),
                        );
                    }
                }
            })
    }
}

// -- Cohen-Sutherland line clipping --

const INSIDE: u8 = 0;
const LEFT: u8 = 1;
const RIGHT: u8 = 2;
const BOTTOM: u8 = 4;
const TOP: u8 = 8;

fn outcode(x: f64, y: f64, x_bounds: &[f64; 2], y_bounds: &[f64; 2]) -> u8 {
    let mut code = INSIDE;
    if x < x_bounds[0] {
        code |= LEFT;
    } else if x > x_bounds[1] {
        code |= RIGHT;
    }
    if y < y_bounds[0] {
        code |= BOTTOM;
    } else if y > y_bounds[1] {
        code |= TOP;
    }
    code
}

/// Clip a line segment to the rectangle defined by x_bounds and y_bounds.
/// Returns Some((x1, y1, x2, y2)) if any portion is visible, None if fully outside.
fn clip_line(
    mut x1: f64,
    mut y1: f64,
    mut x2: f64,
    mut y2: f64,
    x_bounds: &[f64; 2],
    y_bounds: &[f64; 2],
) -> Option<(f64, f64, f64, f64)> {
    let mut code1 = outcode(x1, y1, x_bounds, y_bounds);
    let mut code2 = outcode(x2, y2, x_bounds, y_bounds);

    loop {
        if (code1 | code2) == 0 {
            // Both inside
            return Some((x1, y1, x2, y2));
        }
        if (code1 & code2) != 0 {
            // Both outside same side
            return None;
        }

        // Pick the point that is outside
        let code_out = if code1 != 0 { code1 } else { code2 };
        let dx = x2 - x1;
        let dy = y2 - y1;

        let (x, y);
        if code_out & TOP != 0 {
            x = x1 + dx * (y_bounds[1] - y1) / dy;
            y = y_bounds[1];
        } else if code_out & BOTTOM != 0 {
            x = x1 + dx * (y_bounds[0] - y1) / dy;
            y = y_bounds[0];
        } else if code_out & RIGHT != 0 {
            y = y1 + dy * (x_bounds[1] - x1) / dx;
            x = x_bounds[1];
        } else {
            y = y1 + dy * (x_bounds[0] - x1) / dx;
            x = x_bounds[0];
        }

        if code_out == code1 {
            x1 = x;
            y1 = y;
            code1 = outcode(x1, y1, x_bounds, y_bounds);
        } else {
            x2 = x;
            y2 = y;
            code2 = outcode(x2, y2, x_bounds, y_bounds);
        }
    }
}

// -- Labels --

struct Label {
    x: f64,
    y: f64,
    text: String,
    color: Color,
}

fn collect_labels(
    doc: &KmlDocument,
    viewport: &Viewport,
    selected_path: &Option<Vec<usize>>,
) -> Vec<Label> {
    let mut labels = Vec::new();
    for (i, feature) in doc.features.iter().enumerate() {
        collect_feature_labels(feature, viewport, &[i], selected_path, &mut labels);
    }
    labels
}

fn collect_feature_labels(
    feature: &crate::model::Feature,
    viewport: &Viewport,
    path: &[usize],
    selected_path: &Option<Vec<usize>>,
    labels: &mut Vec<Label>,
) {
    match feature {
        crate::model::Feature::Folder { features, .. } => {
            for (i, child) in features.iter().enumerate() {
                let mut child_path = path.to_vec();
                child_path.push(i);
                collect_feature_labels(child, viewport, &child_path, selected_path, labels);
            }
        }
        crate::model::Feature::Placemark { name, geometry, .. } => {
            if let Some(geom) = geometry {
                if let Some(coord) = label_coord(geom) {
                    let is_selected = selected_path.as_ref().map(|sp| sp == path).unwrap_or(false);
                    let color = if is_selected {
                        Color::Yellow
                    } else {
                        Color::White
                    };
                    let (x, y) = viewport.project_for_canvas(&coord);
                    labels.push(Label {
                        x,
                        y,
                        text: name.clone(),
                        color,
                    });
                }
            }
        }
    }
}

/// Pick a representative coordinate for label placement.
fn label_coord(geom: &Geometry) -> Option<crate::model::Coord> {
    match geom {
        Geometry::Point(c) => Some(c.clone()),
        Geometry::LineString(cs) => {
            // Middle of the line
            if cs.is_empty() {
                return None;
            }
            Some(cs[cs.len() / 2].clone())
        }
        Geometry::Polygon(rings) => {
            // Centroid approximation: average of outer ring
            let ring = rings.first()?;
            if ring.is_empty() {
                return None;
            }
            let n = ring.len() as f64;
            let lon = ring.iter().map(|c| c.lon).sum::<f64>() / n;
            let lat = ring.iter().map(|c| c.lat).sum::<f64>() / n;
            Some(crate::model::Coord {
                lon,
                lat,
                alt: None,
            })
        }
        Geometry::MultiGeometry(gs) => gs.first().and_then(label_coord),
    }
}

// -- Draw commands --

enum DrawCmd {
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        color: Color,
    },
    Point {
        x: f64,
        y: f64,
        color: Color,
    },
}

fn collect_segments(
    doc: &KmlDocument,
    viewport: &Viewport,
    selected_path: &Option<Vec<usize>>,
) -> Vec<DrawCmd> {
    let mut cmds = Vec::new();
    for (i, feature) in doc.features.iter().enumerate() {
        collect_feature_segments(feature, viewport, &[i], selected_path, &mut cmds);
    }
    cmds
}

fn collect_feature_segments(
    feature: &crate::model::Feature,
    viewport: &Viewport,
    path: &[usize],
    selected_path: &Option<Vec<usize>>,
    cmds: &mut Vec<DrawCmd>,
) {
    match feature {
        crate::model::Feature::Folder { features, .. } => {
            for (i, child) in features.iter().enumerate() {
                let mut child_path = path.to_vec();
                child_path.push(i);
                collect_feature_segments(child, viewport, &child_path, selected_path, cmds);
            }
        }
        crate::model::Feature::Placemark { geometry, .. } => {
            if let Some(geom) = geometry {
                let is_selected = selected_path.as_ref().map(|sp| sp == path).unwrap_or(false);
                let color = if is_selected {
                    Color::Yellow
                } else {
                    Color::White
                };
                collect_geom_segments(geom, viewport, color, cmds);
            }
        }
    }
}

fn collect_geom_segments(
    geom: &Geometry,
    viewport: &Viewport,
    color: Color,
    cmds: &mut Vec<DrawCmd>,
) {
    match geom {
        Geometry::Point(coord) => {
            let (x, y) = viewport.project_for_canvas(coord);
            cmds.push(DrawCmd::Point { x, y, color });
        }
        Geometry::LineString(coords) => {
            for window in coords.windows(2) {
                let (x1, y1) = viewport.project_for_canvas(&window[0]);
                let (x2, y2) = viewport.project_for_canvas(&window[1]);
                cmds.push(DrawCmd::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    color,
                });
            }
        }
        Geometry::Polygon(rings) => {
            for ring in rings {
                for window in ring.windows(2) {
                    let (x1, y1) = viewport.project_for_canvas(&window[0]);
                    let (x2, y2) = viewport.project_for_canvas(&window[1]);
                    cmds.push(DrawCmd::Line {
                        x1,
                        y1,
                        x2,
                        y2,
                        color,
                    });
                }
            }
        }
        Geometry::MultiGeometry(geoms) => {
            for g in geoms {
                collect_geom_segments(g, viewport, color, cmds);
            }
        }
    }
}
