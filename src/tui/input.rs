use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Tree,
    Map,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    MoveUp,
    MoveDown,
    ToggleExpand,
    ToggleTree,
    ZoomIn,
    ZoomOut,
    PanLeft,
    PanRight,
    PanUp,
    PanDown,
    None,
}

pub fn handle_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,

        // Tree navigation — arrow keys
        KeyCode::Up => Action::MoveUp,
        KeyCode::Down => Action::MoveDown,
        KeyCode::Enter => Action::ToggleExpand,

        // Map pan — hjkl (vim-style)
        KeyCode::Char('h') => Action::PanLeft,
        KeyCode::Char('j') => Action::PanDown,
        KeyCode::Char('k') => Action::PanUp,
        KeyCode::Char('l') => Action::PanRight,
        KeyCode::Left => Action::PanLeft,
        KeyCode::Right => Action::PanRight,

        // Map zoom
        KeyCode::Char('+') | KeyCode::Char('=') => Action::ZoomIn,
        KeyCode::Char('-') => Action::ZoomOut,

        // Toggle tree panel
        KeyCode::Char('t') | KeyCode::Tab => Action::ToggleTree,

        _ => Action::None,
    }
}

pub fn handle_mouse(mouse: MouseEvent) -> Action {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            if mouse.modifiers.contains(KeyModifiers::SHIFT) {
                Action::PanLeft
            } else {
                Action::ZoomIn
            }
        }
        MouseEventKind::ScrollDown => {
            if mouse.modifiers.contains(KeyModifiers::SHIFT) {
                Action::PanRight
            } else {
                Action::ZoomOut
            }
        }
        MouseEventKind::ScrollLeft => Action::PanLeft,
        MouseEventKind::ScrollRight => Action::PanRight,
        _ => Action::None,
    }
}
