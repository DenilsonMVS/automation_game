use crate::ui::{MouseAction, events::{ActionType, Events}};
use winit::window::{CursorIcon, Window};

pub struct AreaConflict {
    pub id: u32,
    pub depth: u16,
    pub cursor: CursorIcon,
}

pub fn resolve_multi_area_conflict(
    mouse_action: Option<MouseAction>,
    conflicts: &[AreaConflict],
    window: &Window,
) -> Events {
    let winner = conflicts.iter().min_by_key(|area| area.depth);
    window.set_cursor(winner.map(|w| w.cursor).unwrap_or(CursorIcon::Default));

    Events {
        mouse_actions: match (mouse_action, winner) {
            (Some(MouseAction::Pressed), None) => ActionType::NonePress,
            (Some(MouseAction::Pressed), Some(area)) => ActionType::AreaPress(area.id),
            (Some(MouseAction::Released), None) => ActionType::NoneRelease,
            (Some(MouseAction::Released), Some(area)) => ActionType::AreaRelease(area.id),
            (None, None) => ActionType::NoneHover,
            (None, Some(area)) => ActionType::AreaHover(area.id),
        }
    }
}
