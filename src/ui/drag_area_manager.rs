use winit::window::{Cursor, CursorIcon, Window};
use bitvec::prelude::*;
use crate::{game::scene::EventType, stable_index_vector::StableIndexVector, ui::drag_area::DragArea};
use nalgebra_glm as glm;

pub struct DragAreaManager {
    drag_areas: StableIndexVector<DragArea>,
    liveness: BitVec<usize, Lsb0>, 
    grabbed_id: Option<usize>,
}

impl DragAreaManager {
    pub fn new() -> Self {
        Self { 
            drag_areas: StableIndexVector::new(), 
            liveness: BitVec::new(),
            grabbed_id: None 
        }
    }

    fn ensure_liveness_capacity(&mut self, id: usize) {
        if id >= self.liveness.len() {
            self.liveness.resize(id + 1, false);
        }
    }

    pub fn push_drag_area(&mut self, drag_area: DragArea) -> usize {
        let id = self.drag_areas.push(drag_area);
        self.keep_alive(id);
        id
    }

    pub fn update_drag_area(&mut self, id: usize, drag_area: DragArea) {
        if let Some(area) = self.drag_areas.get_mut(id) {
            *area = drag_area;
            self.keep_alive(id);
        }
    }

    pub fn keep_alive(&mut self, id: usize) {
        self.ensure_liveness_capacity(id);
        self.liveness.set(id, true);
    }

    pub fn get_drag_area(&mut self, id: usize) -> Option<&DragArea> {
        if self.drag_areas.get(id).is_some() {
            self.keep_alive(id);
        }

        self.drag_areas.get(id)
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
        // Reset all bits to false (dead) at the start of the frame.
        // This is very fast (SoA optimization).
        self.liveness.fill(false);

        let mut hovering = false;
        // Check hover state for all active areas
        for area in self.drag_areas.iter_unordered() {
            if area.bounding_box.is_point_inside(ndc_mouse_position) {
                hovering = true;
                break;
            }
        }

        if mouse_pressed {
            for (id, area) in self.drag_areas.iter_enumerate_unordered() {
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

        // Mouse movement and cursor management
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
        // Find IDs that weren't "kept alive" this frame.
        // We use BitVec to check the state without touching the DragArea data.
        let dead_ids: Vec<usize> = self.drag_areas
            .iter_enumerate_unordered()
            .filter_map(|(id, _)| {
                // If ID is out of liveness bounds or the bit is false, it's dead.
                let is_alive = self.liveness.get(id).map(|b| *b).unwrap_or(false);
                if !is_alive { Some(id) } else { None }
            })
            .collect();
        
        for dead_id in dead_ids {
            self.drag_areas.remove(dead_id);
        }
    }
}
