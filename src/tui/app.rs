use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Frame,
};

use crate::{
    model::{Feature, Geometry, KmlDocument},
    projection::Viewport,
    tui::{
        details::DetailsView,
        input::{handle_key, Action, Focus},
        map::MapView,
        tree::{kind_to_icon, TreeView, TreeViewItem},
    },
};

#[derive(Debug, Clone)]
pub struct TreeItem {
    pub depth: usize,
    pub name: String,
    pub kind: String,
    pub expanded: bool,
    pub has_children: bool,
    pub feature_path: Vec<usize>,
}

pub struct App {
    doc: KmlDocument,
    viewport: Viewport,
    focus: Focus,
    selected: usize,
    tree_items: Vec<TreeItem>,
    tree_scroll: usize,
    should_quit: bool,
}

impl App {
    pub fn new(doc: KmlDocument) -> Self {
        let viewport = doc
            .bounding_box()
            .map(|bb| Viewport::from_bbox(&bb))
            .unwrap_or_else(|| {
                Viewport::from_bbox(&crate::model::BoundingBox {
                    min_lon: -180.0,
                    max_lon: 180.0,
                    min_lat: -90.0,
                    max_lat: 90.0,
                })
            });

        let tree_items = build_tree_items(&doc.features, 0, &[]);

        Self {
            doc,
            viewport,
            focus: Focus::Tree,
            selected: 0,
            tree_items,
            tree_scroll: 0,
            should_quit: false,
        }
    }

    pub fn run(mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stderr = std::io::stderr();
        execute!(stderr, EnterAlternateScreen)?;

        let backend = ratatui::backend::CrosstermBackend::new(stderr);
        let mut terminal = ratatui::Terminal::new(backend)?;

        let result = self.event_loop(&mut terminal);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    fn event_loop(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>,
    ) -> Result<()> {
        loop {
            terminal.draw(|f| self.draw(f))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    let action = handle_key(key, self.focus);
                    self.handle_action(action);
                }
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::SwitchFocus => {
                self.focus = match self.focus {
                    Focus::Tree => Focus::Map,
                    Focus::Map => Focus::Tree,
                };
            }
            Action::MoveDown => {
                let visible: Vec<usize> = self.visible_indices();
                if let Some(pos) = visible.iter().position(|&i| i == self.selected) {
                    if pos + 1 < visible.len() {
                        self.selected = visible[pos + 1];
                        // scroll down if needed (use 20 as default visible height)
                        let new_pos = pos + 1;
                        if new_pos >= self.tree_scroll + 20 {
                            self.tree_scroll = new_pos.saturating_sub(19);
                        }
                    }
                } else if !visible.is_empty() {
                    self.selected = visible[0];
                    self.tree_scroll = 0;
                }
            }
            Action::MoveUp => {
                let visible: Vec<usize> = self.visible_indices();
                if let Some(pos) = visible.iter().position(|&i| i == self.selected) {
                    if pos > 0 {
                        self.selected = visible[pos - 1];
                        let new_pos = pos - 1;
                        if new_pos < self.tree_scroll {
                            self.tree_scroll = new_pos;
                        }
                    }
                } else if !visible.is_empty() {
                    self.selected = visible[0];
                    self.tree_scroll = 0;
                }
            }
            Action::ToggleExpand => {
                if let Some(item) = self.tree_items.get(self.selected).cloned() {
                    if item.has_children {
                        self.tree_items[self.selected].expanded =
                            !self.tree_items[self.selected].expanded;
                    } else {
                        // Leaf: center viewport on first coord
                        if let Some(coord) = self.first_coord_of_selected() {
                            self.viewport.center_on(&coord);
                        }
                    }
                }
            }
            Action::ZoomIn => self.viewport.zoom_in(),
            Action::ZoomOut => self.viewport.zoom_out(),
            Action::PanLeft => self.viewport.pan_left(),
            Action::PanRight => self.viewport.pan_right(),
            Action::PanUp => self.viewport.pan_up(),
            Action::PanDown => self.viewport.pan_down(),
            Action::Search | Action::None => {}
        }
    }

    fn get_feature(&self, path: &[usize]) -> Option<&Feature> {
        let mut features = &self.doc.features;
        let mut feature = None;
        for &idx in path {
            feature = features.get(idx);
            if let Some(Feature::Folder {
                features: children, ..
            }) = feature
            {
                features = children;
            } else {
                break;
            }
        }
        feature
    }

    fn first_coord_of_selected(&self) -> Option<crate::model::Coord> {
        let path = self.tree_items.get(self.selected)?.feature_path.clone();
        let feature = self.get_feature(&path)?;
        match feature {
            Feature::Placemark {
                geometry: Some(geom),
                ..
            } => first_coord(geom),
            _ => None,
        }
    }

    fn visible_indices(&self) -> Vec<usize> {
        let mut visible = Vec::new();
        let mut i = 0;
        while i < self.tree_items.len() {
            visible.push(i);
            let item = &self.tree_items[i];
            if item.has_children && !item.expanded {
                // skip children
                let depth = item.depth;
                i += 1;
                while i < self.tree_items.len() && self.tree_items[i].depth > depth {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        visible
    }

    fn draw(&self, f: &mut Frame) {
        let area = f.area();

        // Split: body + details + status
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(area);

        let body_area = outer[0];
        let details_area = outer[1];
        let status_area = outer[2];

        // Split body: tree 30% + map 70%
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(body_area);

        let tree_area = body[0];
        let map_area = body[1];

        // Tree panel
        let tree_border_style = if self.focus == Focus::Tree {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let tree_block = Block::default()
            .borders(Borders::ALL)
            .title("Features")
            .border_style(tree_border_style);
        let tree_inner = tree_block.inner(tree_area);
        f.render_widget(tree_block, tree_area);

        let visible_indices = self.visible_indices();
        let view_items: Vec<TreeViewItem> = visible_indices
            .iter()
            .map(|&idx| {
                let item = &self.tree_items[idx];
                TreeViewItem {
                    depth: item.depth,
                    name: item.name.clone(),
                    icon: kind_to_icon(&item.kind),
                    expanded: item.expanded,
                    has_children: item.has_children,
                }
            })
            .collect();

        // selected is an index into tree_items; find its position in visible list
        let selected_pos = visible_indices
            .iter()
            .position(|&i| i == self.selected)
            .unwrap_or(0);

        f.render_widget(
            TreeView::new(&view_items, selected_pos, self.tree_scroll),
            tree_inner,
        );

        // Map panel
        let selected_path = self
            .tree_items
            .get(self.selected)
            .map(|i| i.feature_path.as_slice());
        let map_view = MapView::new(
            &self.doc,
            &self.viewport,
            selected_path,
            self.focus == Focus::Map,
        );
        f.render_widget(map_view.widget(), map_area);

        // Details panel
        let selected_feature = self
            .tree_items
            .get(self.selected)
            .and_then(|item| self.get_feature(&item.feature_path));
        let details_view = DetailsView::new(selected_feature);
        f.render_widget(details_view.widget(), details_area);

        // Status bar
        let status = Paragraph::new(Span::raw(
            " [Tab] Switch focus  [q] Quit  [j/k] Navigate  [Enter] Expand  [+/-] Zoom  [hjkl] Pan",
        ))
        .style(Style::default().fg(Color::DarkGray));
        f.render_widget(status, status_area);
    }
}

fn feature_kind(feature: &Feature) -> &'static str {
    match feature {
        Feature::Folder { .. } => "folder",
        Feature::Placemark { geometry, .. } => match geometry {
            Some(crate::model::Geometry::Point(_)) => "point",
            Some(crate::model::Geometry::LineString(_)) => "line",
            Some(crate::model::Geometry::Polygon(_)) => "polygon",
            Some(crate::model::Geometry::MultiGeometry(_)) => "multi",
            None => "placemark",
        },
    }
}

fn first_coord(geom: &Geometry) -> Option<crate::model::Coord> {
    match geom {
        Geometry::Point(c) => Some(c.clone()),
        Geometry::LineString(cs) => cs.first().cloned(),
        Geometry::Polygon(rings) => rings.first()?.first().cloned(),
        Geometry::MultiGeometry(gs) => gs.first().and_then(first_coord),
    }
}

fn build_tree_items(features: &[Feature], depth: usize, parent_path: &[usize]) -> Vec<TreeItem> {
    let mut items = Vec::new();
    for (i, feature) in features.iter().enumerate() {
        let mut path = parent_path.to_vec();
        path.push(i);

        let (name, has_children) = match feature {
            Feature::Folder { name, features } => (name.clone(), !features.is_empty()),
            Feature::Placemark { name, .. } => (name.clone(), false),
        };

        items.push(TreeItem {
            depth,
            name,
            kind: feature_kind(feature).to_string(),
            expanded: depth == 0,
            has_children,
            feature_path: path.clone(),
        });

        if let Feature::Folder {
            features: children, ..
        } = feature
        {
            let child_items = build_tree_items(children, depth + 1, &path);
            items.extend(child_items);
        }
    }
    items
}
