use crate::{game::scene::EventType, ui::drag_area_manager::DragAreaManager};
use nalgebra_glm as glm;
use winit::window::{CursorIcon, Window};

#[derive(Debug)]
pub enum Area {
    SimpleArea(usize, u16, bool),
    DragArea(usize, u16),
}

impl Area {
    fn get_depth(&self) -> u16 {
        match self {
            Self::DragArea(_, depth) => *depth,
            Self::SimpleArea(_, depth, _) => *depth,
        }
    }
}

pub fn resolve_multi_area_conflict(
    mouse_pressed: bool,
    mouse_released: bool,
    ndc_mouse_position: glm::Vec2,
    previous_ndc_mouse_position: glm::Vec2,
    conflicts: &[Area],
    events: &mut Vec<EventType>,
    drag_area_manager: &mut DragAreaManager,
    window: &Window,
) {
    if let Some(id) = drag_area_manager.grabbed_id {
        if mouse_released {
            events.push(EventType::Released(id));
            drag_area_manager.grabbed_id = None;
            window.set_cursor(CursorIcon::Grabbing);
        } else {
            let delta = ndc_mouse_position - previous_ndc_mouse_position;
            if delta != glm::vec2(0.0, 0.0) {
                events.push(EventType::Moved(Box::new((id, delta))));
            }
            window.set_cursor(CursorIcon::Grabbing);
        }
        return;
    }

    let winner = conflicts.iter().min_by_key(|area| area.get_depth());

    if let Some(winning_area) = winner {
        match winning_area {
            Area::DragArea(id, _) => {
                window.set_cursor(CursorIcon::Grab);
                events.push(EventType::HoverDragArea(*id));
            }
            Area::SimpleArea(id, _, is_button) => {
                window.set_cursor(if *is_button { CursorIcon::Pointer } else { CursorIcon::Default });
                events.push(EventType::HoverSimpleArea(*id));
            }
        }
    } else {
        if mouse_pressed {
            events.push(EventType::ClickOutside);
        }
        window.set_cursor(CursorIcon::Default);
    }

    if let (Some(winning_area), true) = (winner, mouse_pressed) {
        match winning_area {
            Area::SimpleArea(id, _, _) => {
                events.push(EventType::SimpleAreaPressed(*id));
            }
            Area::DragArea(id, _) => {
                drag_area_manager.grabbed_id = Some(*id);
                events.push(EventType::Grabbed(*id));
                window.set_cursor(CursorIcon::Grabbing);
            }
        }
    }
}
