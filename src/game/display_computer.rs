use crate::graphics::renderer::fonts::{Font, TextRenderConfig};
use nalgebra_glm as glm;

#[derive(Debug, Clone, Copy)]
pub struct DisplayComputer {
    received_value: Option<u8>,
    value: Option<u8>,
}

impl DisplayComputer {
    pub fn new() -> Self {
        Self { received_value: None, value: None }
    }

    pub fn receive_syscall(&mut self, value: Option<u8>) {
        self.received_value = value;
    }

    pub fn update(&mut self, is_update_tick: bool) -> Option<u8> {
        if is_update_tick {
            self.value = self.received_value;
            Some(1)
        } else {
            None
        }
    }

    pub fn send(&self, font: &mut Font, position: glm::Vec2) {
        if let Some(value) = self.value {
            let value = format!("{}", value);
            font.add_text(&TextRenderConfig::new(
                &value,
                position,
                0.5,
                glm::vec4(20, 45, 90, 255),
            ).set_depth(0));
        }
    }
}