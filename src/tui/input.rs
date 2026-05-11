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
    SwitchFocus,
    ZoomIn,
    ZoomOut,
    PanLeft,
    PanRight,
    PanUp,
    PanDown,
    Search,
    None,
}

pub fn handle_key(key: KeyEvent, focus: Focus) -> Action {
    // Global bindings
    match key.code {
        KeyCode::Char('q') => return Action::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Action::Quit,
        KeyCode::Tab => return Action::SwitchFocus,
        KeyCode::Char('/') => return Action::Search,
        _ => {}
    }

    match focus {
        Focus::Tree => match key.code {
            KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
            KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
            KeyCode::Enter => Action::ToggleExpand,
            _ => Action::None,
        },
        Focus::Map => match key.code {
            KeyCode::Char('h') | KeyCode::Left => Action::PanLeft,
            KeyCode::Char('l') | KeyCode::Right => Action::PanRight,
            KeyCode::Char('k') | KeyCode::Up => Action::PanUp,
            KeyCode::Char('j') | KeyCode::Down => Action::PanDown,
            KeyCode::Char('+') => Action::ZoomIn,
            KeyCode::Char('-') => Action::ZoomOut,
            _ => Action::None,
        },
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
