use crate::{game::scene::EventType, stable_index_vector::StableIndexVector, ui::{UiContext, button_area::ButtonArea}};
use nalgebra_glm as glm;
use winit::window::{Cursor, CursorIcon, Window};

pub struct ButtonManager {
    button_areas: StableIndexVector<(ButtonArea, bool)>,
}

impl ButtonManager {
    pub fn new() -> Self {
        Self {
            button_areas: StableIndexVector::new(),
        }
    }

    pub fn push_buton_area(&mut self, button_area: ButtonArea) -> usize {
        self.button_areas.push((button_area, false))
    }

    pub fn update_button_area(&mut self, id: usize, button_area: ButtonArea) {
        if let Some((area, alive)) = self.button_areas.get_mut(id) {
            *area = button_area;
            *alive = true;
        }
    }

    pub(super) fn on_frame_start(
        &mut self,
        events: &mut Vec<EventType>,
        mouse_pressed: bool,
        ndc_mouse_position: glm::Vec2,
        window: &Window,
    ) {
        for (id, (button, alive)) in self.button_areas.iter_enumerate_unordered_mut() {
            *alive = false;
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
            .filter(|(_, (_, alive))| !alive)
            .map(|(id, _)| id)
            .collect();
        
        for dead_id in dead_ids.iter() {
            self.button_areas.remove(*dead_id);
        }
    }
}