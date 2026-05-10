use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

pub struct TreeViewItem {
    pub depth: usize,
    pub name: String,
    pub icon: &'static str,
    pub expanded: bool,
    pub has_children: bool,
}

pub struct TreeView<'a> {
    pub items: &'a [TreeViewItem],
    pub selected: usize,
    pub scroll_offset: usize,
}

impl<'a> TreeView<'a> {
    pub fn new(items: &'a [TreeViewItem], selected: usize, scroll_offset: usize) -> Self {
        Self {
            items,
            selected,
            scroll_offset,
        }
    }
}

impl Widget for TreeView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let height = area.height as usize;
        let visible = self
            .items
            .iter()
            .skip(self.scroll_offset)
            .take(height)
            .enumerate();

        for (row, item) in visible {
            let y = area.y + row as u16;
            if y >= area.y + area.height {
                break;
            }

            let indent = "  ".repeat(item.depth);
            let expand_marker = if item.has_children {
                if item.expanded {
                    "▼ "
                } else {
                    "▶ "
                }
            } else {
                "  "
            };

            let line_str = format!("{}{}{} {}", indent, expand_marker, item.icon, item.name);

            let global_index = self.scroll_offset + row;
            let style = if global_index == self.selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let line = Line::from(Span::styled(line_str, style));
            let x = area.x;
            let width = area.width as usize;

            // Truncate to area width
            let content: String = line
                .spans
                .iter()
                .flat_map(|s| s.content.chars())
                .take(width)
                .collect();

            buf.set_string(x, y, &content, style);
        }
    }
}

pub fn kind_to_icon(kind: &str) -> &'static str {
    match kind {
        "folder" => "📁",
        "point" => "●",
        "line" => "─",
        "polygon" => "◻",
        "multi" => "◈",
        _ => "·",
    }
}
