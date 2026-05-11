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

        // Tree navigation — always available
        KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
        KeyCode::Enter => Action::ToggleExpand,

        // Map controls — always available
        KeyCode::Char('h') | KeyCode::Left => Action::PanLeft,
        KeyCode::Char('l') | KeyCode::Right => Action::PanRight,
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
