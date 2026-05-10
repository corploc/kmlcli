use ratatui::{
    style::Color,
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Line as CanvasLine, Points},
        Block, Borders,
    },
};

use crate::{
    model::{Geometry, KmlDocument},
    projection::Viewport,
};

pub struct MapView<'a> {
    pub doc: &'a KmlDocument,
    pub viewport: &'a Viewport,
    pub selected_path: Option<&'a [usize]>,
    pub focused: bool,
}

impl<'a> MapView<'a> {
    pub fn new(
        doc: &'a KmlDocument,
        viewport: &'a Viewport,
        selected_path: Option<&'a [usize]>,
        focused: bool,
    ) -> Self {
        Self {
            doc,
            viewport,
            selected_path,
            focused,
        }
    }

    pub fn widget(self) -> impl ratatui::widgets::Widget + 'a {
        let x_bounds = self.viewport.x_bounds();
        let y_bounds = self.viewport.y_bounds();

        let border_style = if self.focused {
            ratatui::style::Style::default().fg(Color::Yellow)
        } else {
            ratatui::style::Style::default()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Map")
            .border_style(border_style);

        // Collect geometry segments to draw
        // We snapshot what we need so the closure can own it
        let selected_path = self.selected_path.map(|p| p.to_vec());
        let segments = collect_segments(self.doc, self.viewport, &selected_path);

        Canvas::default()
            .block(block)
            .x_bounds(x_bounds)
            .y_bounds(y_bounds)
            .marker(Marker::Braille)
            .paint(move |ctx| {
                for seg in &segments {
                    match seg {
                        DrawCmd::Line {
                            x1,
                            y1,
                            x2,
                            y2,
                            color,
                        } => {
                            ctx.draw(&CanvasLine {
                                x1: *x1,
                                y1: *y1,
                                x2: *x2,
                                y2: *y2,
                                color: *color,
                            });
                        }
                        DrawCmd::Point { x, y, color } => {
                            let coords = [(*x, *y)];
                            ctx.draw(&Points {
                                coords: &coords,
                                color: *color,
                            });
                        }
                    }
                }
            })
    }
}

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
                    Color::DarkGray
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
