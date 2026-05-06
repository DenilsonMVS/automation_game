use crate::ui::{MouseAction, events::{ActionType, Events}, simple_area_manager::AreaId};
use winit::window::{CursorIcon, Window};

#[derive(Debug)]
pub struct AreaConflict {
    pub id: AreaId,
    pub depth: u16,
    pub cursor: CursorIcon,
    pub scroll: bool,
}

fn resolve_scroll(conflicts: &[AreaConflict]) -> AreaId {
    let scroll_winner = conflicts.iter().filter(|area| area.scroll).min_by_key(|area| area.depth);
    scroll_winner.map(|w| w.id).unwrap_or(AreaId::default())
}

fn resolve_mouse_actions(
    mouse_action: Option<MouseAction>,
    conflicts: &[AreaConflict],
    window: &Window,
    prev_actions: &ActionType,
) -> ActionType {
    let winner = conflicts.iter().min_by_key(|area| area.depth);
    if let Some(winner) = winner && winner.cursor == CursorIcon::Grab && mouse_action == Some(MouseAction::Pressed) {
        window.set_cursor(CursorIcon::Grabbing);
        return ActionType::Grabbing(winner.id, winner.depth);
    } else if let ActionType::Grabbing(_, _) = prev_actions && mouse_action != Some(MouseAction::Released) {
        window.set_cursor(CursorIcon::Grabbing);
        return *prev_actions;
    }

    window.set_cursor(winner.map(|w| w.cursor).unwrap_or(CursorIcon::Default));
    match (mouse_action, winner) {
        (Some(MouseAction::Pressed), None) => ActionType::NonePress,
        (Some(MouseAction::Pressed), Some(area)) => ActionType::AreaPress(area.id, area.depth),
        (Some(MouseAction::Released), None) => ActionType::NoneRelease,
        (Some(MouseAction::Released), Some(area)) => ActionType::AreaRelease(area.id, area.depth),
        (None, None) => ActionType::NoneHover,
        (None, Some(area)) => ActionType::AreaHover(area.id, area.depth),
    }
}

pub fn resolve_multi_area_conflict(
    mouse_action: Option<MouseAction>,
    conflicts: &[AreaConflict],
    window: &Window,
    prev_events: &Events,
) -> Events {
    Events {
        mouse_actions: resolve_mouse_actions(mouse_action, conflicts, window, &prev_events.mouse_actions),
        scroll: resolve_scroll(conflicts),
    }
}
