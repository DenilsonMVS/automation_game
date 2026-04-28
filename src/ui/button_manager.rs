use crate::{game::scene::EventType, stable_index_vector::StableIndexVector, ui::button_area::ButtonArea};
use bitvec::{order::Lsb0, vec::BitVec};
use nalgebra_glm as glm;
use winit::window::{Cursor, CursorIcon, Window};

pub struct ButtonManager {
    button_areas: StableIndexVector<ButtonArea>,
    liveness: BitVec<usize, Lsb0>, 
}

impl ButtonManager {
    pub fn new() -> Self {
        Self {
            button_areas: StableIndexVector::new(),
            liveness: BitVec::new(),
        }
    }

    fn ensure_liveness_capacity(&mut self, id: usize) {
        if id >= self.liveness.len() {
            self.liveness.resize(id + 1, false);
        }
    }

    pub fn keep_alive(&mut self, id: usize) {
        self.ensure_liveness_capacity(id);
        self.liveness.set(id, true);
    }

    pub fn push_buton_area(&mut self, button_area: ButtonArea) -> usize {
        let id = self.button_areas.push(button_area);
        self.keep_alive(id);
        id
    }

    pub fn update_button_area(&mut self, id: usize, button_area: ButtonArea) {
        if let Some(area) = self.button_areas.get_mut(id) {
            *area = button_area;
            self.keep_alive(id);
        }
    }

    pub(super) fn on_frame_start(
        &mut self,
        events: &mut Vec<EventType>,
        mouse_pressed: bool,
        ndc_mouse_position: glm::Vec2,
        window: &Window,
    ) {
        self.liveness.fill(false);

        for (id, button) in self.button_areas.iter_enumerate_unordered_mut() {
            if button.bounding_box.is_point_inside(ndc_mouse_position) {
                window.set_cursor(Cursor::Icon(CursorIcon::Pointer));
                if mouse_pressed {
                    events.push(EventType::ButtonPressed(id));
                }
            }
        }
    } 

    pub(super) fn on_frame_end(&mut self) {
        let dead_ids: Vec<usize> = self.button_areas
            .iter_enumerate_unordered()
            .filter_map(|(id, _)| {
                // If ID is out of liveness bounds or the bit is false, it's dead.
                let is_alive = self.liveness.get(id).map(|b| *b).unwrap_or(false);
                if !is_alive { Some(id) } else { None }
            })
            .collect();
        
        for dead_id in dead_ids.iter() {
            self.button_areas.remove(*dead_id);
        }
    }
}