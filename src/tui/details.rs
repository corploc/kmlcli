use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::model::{Feature, Geometry};

pub struct DetailsView<'a> {
    pub feature: Option<&'a Feature>,
}

impl<'a> DetailsView<'a> {
    pub fn new(feature: Option<&'a Feature>) -> Self {
        Self { feature }
    }

    pub fn widget(self) -> impl Widget + 'a {
        let block = Block::default().borders(Borders::ALL).title("Details");

        let lines: Vec<Line> = match self.feature {
            None => vec![Line::from(Span::styled(
                "No selection",
                Style::default().fg(Color::DarkGray),
            ))],
            Some(Feature::Folder { name, features }) => {
                let bold = Style::default().add_modifier(Modifier::BOLD);
                vec![
                    Line::from(vec![
                        Span::styled("📁 ", Style::default()),
                        Span::styled(name.as_str(), bold),
                    ]),
                    Line::from(Span::styled(
                        format!("{} item(s)", features.len()),
                        Style::default().fg(Color::Gray),
                    )),
                ]
            }
            Some(Feature::Placemark {
                name,
                geometry,
                description,
                style_id,
            }) => {
                let bold = Style::default().add_modifier(Modifier::BOLD);
                let mut lines = Vec::new();

                // Name + geometry icon
                let icon = match geometry {
                    Some(Geometry::Point(_)) => "● ",
                    Some(Geometry::LineString(_)) => "─ ",
                    Some(Geometry::Polygon(_)) => "◻ ",
                    Some(Geometry::MultiGeometry(_)) => "◈ ",
                    None => "· ",
                };
                lines.push(Line::from(vec![
                    Span::styled(icon, Style::default().fg(Color::Yellow)),
                    Span::styled(name.as_str(), bold),
                ]));

                // Geometry info
                if let Some(geom) = geometry {
                    lines.push(Line::from(Span::styled(
                        geom_info(geom),
                        Style::default().fg(Color::Gray),
                    )));
                }

                // Description (truncated)
                if let Some(desc) = description {
                    let short: String = desc.chars().take(60).collect();
                    let suffix = if desc.len() > 60 { "…" } else { "" };
                    lines.push(Line::from(Span::styled(
                        format!("{short}{suffix}"),
                        Style::default().fg(Color::DarkGray),
                    )));
                }

                // Style id
                if let Some(sid) = style_id {
                    lines.push(Line::from(Span::styled(
                        format!("style: {sid}"),
                        Style::default().fg(Color::DarkGray),
                    )));
                }

                lines
            }
        };

        Paragraph::new(lines).block(block)
    }
}

fn geom_info(geom: &Geometry) -> String {
    match geom {
        Geometry::Point(c) => format!("Point ({:.4}, {:.4})", c.lon, c.lat),
        Geometry::LineString(cs) => format!("LineString — {} points", cs.len()),
        Geometry::Polygon(rings) => {
            let pts: usize = rings.iter().map(|r| r.len()).sum();
            format!("Polygon — {} ring(s), {} points", rings.len(), pts)
        }
        Geometry::MultiGeometry(gs) => format!("MultiGeometry — {} geometries", gs.len()),
    }
}
