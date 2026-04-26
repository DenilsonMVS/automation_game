use winit::window::{Cursor, CursorIcon, Window};

use crate::{game::scene::EventType, stable_index_vector::StableIndexVector, ui::drag_area::DragArea};
use nalgebra_glm as glm;

pub struct DragAreaManager {
    drag_areas: StableIndexVector<(DragArea, bool)>,
    grabbed_id: Option<usize>,
}

impl DragAreaManager {
    pub fn new() -> Self {
        Self { drag_areas: StableIndexVector::new(), grabbed_id: None }
    }

    pub fn push_drag_area(&mut self, drag_area: DragArea) -> usize {
        self.drag_areas.push((drag_area, false))
    }

    pub fn update_drag_area(&mut self, id: usize, drag_area: DragArea) {
        if let Some((area, alive)) = self.drag_areas.get_mut(id) {
            *area = drag_area;
            *alive = true; 
        }
    }

    pub fn keep_alive(&mut self, id: usize) {
        if let Some((_, alive)) = self.drag_areas.get_mut(id) {
            *alive = true;
        }
    }

    pub fn get_drag_area(&mut self, id: usize) -> Option<&DragArea> {
        if let Some((area, alive)) = self.drag_areas.get_mut(id) {
            *alive = true;
            Some(area)
        } else {
            None
        }
    }

    pub(super) fn on_frame_start(
        &mut self,
        events: &mut Vec<EventType>,
        mouse_pressed: bool,
        mouse_released: bool,
        previous_ndc_mouse_position: glm::Vec2,
        ndc_mouse_position: glm::Vec2,
        window: &Window,
    ) {
        let mut hovering = false;
        for (area, alive) in self.drag_areas.iter_unordered_mut() {
            *alive = false;
            hovering |= area.bounding_box.is_point_inside(ndc_mouse_position);
        }

        if mouse_pressed {
            for (id, (area, _)) in self.drag_areas.iter_enumerate_unordered_mut() {
                if area.bounding_box.is_point_inside(ndc_mouse_position) {
                    self.grabbed_id = Some(id);
                    events.push(EventType::Grabbed(id));
                    break;
                }
            }
        }
        
        if mouse_released {
            if let Some(id) = self.grabbed_id {
                events.push(EventType::Released(id));
            }
            self.grabbed_id = None;
        }

        if let Some(id) = self.grabbed_id {
            if ndc_mouse_position != previous_ndc_mouse_position {
                events.push(EventType::Moved(Box::new((id, ndc_mouse_position - previous_ndc_mouse_position))));
            }
            window.set_cursor(Cursor::Icon(CursorIcon::Grabbing));
        } else if hovering {
            window.set_cursor(Cursor::Icon(CursorIcon::Grab));
        }
    } 

    pub(super) fn on_frame_end(&mut self) {
        let dead_ids: Vec<usize> = self.drag_areas
            .iter_enumerate_unordered()
            .filter(|(_, (_, alive))| !alive)
            .map(|(id, _)| id)
            .collect();
        
        for dead_id in dead_ids.iter() {
            self.drag_areas.remove(*dead_id);
        }
    }
}
