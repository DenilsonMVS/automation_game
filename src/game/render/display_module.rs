use nalgebra_glm as glm;
use crate::{game::logic::display_module::DisplayModule, graphics::{camera::Camera, renderer::fonts::{Font, TextRenderConfig}}};

impl DisplayModule {
    pub fn send(&self, font: &mut Font, position: glm::Vec2, camera: &Camera) {
        if let Some(value) = self.get_value() {
            let value = format!("{}", value);
            font.add_text(&TextRenderConfig::new(
                &value,
                position,
                0.5,
                glm::vec4(20, 45, 90, 255),
            ).set_depth(0).set_recenter(camera));
        }
    }
}
