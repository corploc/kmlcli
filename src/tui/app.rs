use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    model::{Feature, Geometry, KmlDocument},
    projection::Viewport,
    tiles::fetch::TileCache,
    tui::{
        input::{handle_key, handle_mouse, Action},
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
    selected: usize,
    tree_items: Vec<TreeItem>,
    tree_scroll: usize,
    should_quit: bool,
    tile_cache: TileCache,
    show_tree: bool,
    /// Updated at each render. Fallback (20) is used before the first frame.
    tree_visible_rows: usize,
}

impl App {
    pub fn new(doc: KmlDocument) -> Result<Self> {
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
        let tile_cache = TileCache::new()?;

        let app = Self {
            doc,
            viewport,
            selected: 0,
            tree_items,
            tree_scroll: 0,
            should_quit: false,
            tile_cache,
            show_tree: true,
            tree_visible_rows: 20,
        };
        app.prefetch_visible_tiles();
        Ok(app)
    }

    pub fn run(mut self) -> Result<()> {
        // Install panic hook to restore terminal on crash
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = disable_raw_mode();
            let _ = execute!(std::io::stderr(), LeaveAlternateScreen, DisableMouseCapture);
            original_hook(info);
        }));

        // Signal handler for Ctrl-C (works even if crossterm misses it)
        let should_quit = Arc::new(AtomicBool::new(false));
        {
            let quit = should_quit.clone();
            let _ = ctrlc::set_handler(move || {
                quit.store(true, Ordering::Relaxed);
                // Force restore terminal in case we're stuck
                let _ = disable_raw_mode();
                let _ = execute!(std::io::stderr(), LeaveAlternateScreen, DisableMouseCapture);
            });
        }

        enable_raw_mode()?;
        let mut stderr = std::io::stderr();
        execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = ratatui::backend::CrosstermBackend::new(stderr);
        let mut terminal = ratatui::Terminal::new(backend)?;

        let result = self.event_loop(&mut terminal, &should_quit);

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    fn event_loop(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>,
        signal_quit: &Arc<AtomicBool>,
    ) -> Result<()> {
        loop {
            terminal.draw(|f| self.draw(f))?;

            if signal_quit.load(Ordering::Relaxed) || self.should_quit {
                break;
            }

            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    Event::Key(key) => {
                        let action = handle_key(key);
                        self.handle_action(action);
                    }
                    Event::Mouse(mouse) => {
                        let action = handle_mouse(mouse);
                        self.handle_action(action);
                    }
                    Event::FocusGained => {
                        let _ = enable_raw_mode();
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::ToggleTree => {
                self.show_tree = !self.show_tree;
            }
            Action::MoveDown => {
                let visible: Vec<usize> = self.visible_indices();
                if let Some(pos) = visible.iter().position(|&i| i == self.selected) {
                    if pos + 1 < visible.len() {
                        self.selected = visible[pos + 1];
                        let new_pos = pos + 1;
                        let visible = self.tree_visible_rows.max(1);
                        if new_pos >= self.tree_scroll + visible {
                            self.tree_scroll = new_pos.saturating_sub(visible - 1);
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
                        if let Some(coord) = self.first_coord_of_selected() {
                            self.viewport.center_on(&coord);
                            self.prefetch_visible_tiles();
                        }
                    }
                }
            }
            Action::ZoomIn => {
                self.viewport.zoom_in();
                self.prefetch_visible_tiles();
            }
            Action::ZoomOut => {
                self.viewport.zoom_out();
                self.prefetch_visible_tiles();
            }
            Action::PanLeft => {
                self.viewport.pan_left();
                self.prefetch_visible_tiles();
            }
            Action::PanRight => {
                self.viewport.pan_right();
                self.prefetch_visible_tiles();
            }
            Action::PanUp => {
                self.viewport.pan_up();
                self.prefetch_visible_tiles();
            }
            Action::PanDown => {
                self.viewport.pan_down();
                self.prefetch_visible_tiles();
            }
            Action::None => {}
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

    fn prefetch_visible_tiles(&self) {
        let zoom = self.viewport.zoom_level().min(14);
        let x_bounds = self.viewport.x_bounds();
        let lat_bounds = self.viewport.lat_bounds();
        let mut tiles = crate::tiles::math::visible_tiles(
            lat_bounds[0],
            lat_bounds[1],
            x_bounds[0],
            x_bounds[1],
            zoom,
        );
        tiles.truncate(crate::tiles::math::MAX_VISIBLE_TILES);
        self.tile_cache.prefetch(tiles);
    }

    fn visible_indices(&self) -> Vec<usize> {
        let mut visible = Vec::new();
        let mut i = 0;
        while i < self.tree_items.len() {
            visible.push(i);
            let item = &self.tree_items[i];
            if item.has_children && !item.expanded {
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

    fn draw(&mut self, f: &mut Frame) {
        let area = f.area();

        if area.width < 40 || area.height < 10 {
            let msg = Paragraph::new("Terminal too small (min 40x10)")
                .style(Style::default().fg(Color::Red));
            f.render_widget(msg, area);
            return;
        }

        // Layout: map (full area) + status bar (1 line at bottom)
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);

        let map_area = outer[0];
        let status_area = outer[1];

        // Selected path
        let selected_path = self
            .tree_items
            .get(self.selected)
            .map(|i| i.feature_path.as_slice());

        // Map — fullscreen
        let map_view = MapView::new(
            &self.doc,
            &self.viewport,
            selected_path,
            true,
            &self.tile_cache,
        );
        f.render_widget(map_view.widget(), map_area);

        // Floating tree panel overlay
        if self.show_tree {
            let visible_indices = self.visible_indices();
            let item_count = visible_indices.len();

            // Size the panel to fit content, with limits
            let panel_width = (area.width / 3).clamp(25, 50);
            let panel_height = (item_count as u16 + 2).clamp(4, area.height.saturating_sub(4));
            // Track usable content rows (panel - 2 borders) so scroll math matches reality.
            self.tree_visible_rows = panel_height.saturating_sub(2).max(1) as usize;

            let tree_area = Rect {
                x: 1,
                y: 1,
                width: panel_width,
                height: panel_height,
            };

            // Clear background behind the floating panel
            f.render_widget(Clear, tree_area);

            let tree_block = Block::default()
                .borders(Borders::ALL)
                .title(" Features ")
                .border_style(Style::default().fg(Color::DarkGray));
            let tree_inner = tree_block.inner(tree_area);
            f.render_widget(tree_block, tree_area);

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

            let selected_pos = visible_indices
                .iter()
                .position(|&i| i == self.selected)
                .unwrap_or(0);

            f.render_widget(
                TreeView::new(&view_items, selected_pos, self.tree_scroll),
                tree_inner,
            );
        }

        // Status bar
        let doc_name = self.doc.name.as_deref().unwrap_or("untitled");
        let zoom = self.viewport.zoom_level();
        let status_text = format!(
            " {doc_name} | z{zoom} | [q]uit [t]ree [arrows]tree [hjkl]pan [+/-]zoom [scroll]pan [shift+scroll]h-pan [ctrl+scroll]zoom"
        );
        let status =
            Paragraph::new(Span::raw(status_text)).style(Style::default().fg(Color::DarkGray));
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
