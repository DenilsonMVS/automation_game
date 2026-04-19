use nalgebra_glm as glm;
use crate::{game::{logic::display_module::DisplayModule, render::robot::ROBOT_DEPTH}, graphics::{camera::Camera, renderer::fonts::{Font, TextAlign, TextRenderConfig}}};

const DISPLAY_DEPTH: u16 = ROBOT_DEPTH - 1;

impl DisplayModule {
    pub fn send(&self, font: &mut Font, position: glm::Vec2, camera: &Camera) {
        if let Some(value) = self.get_value() {
            let value = format!("{}", value);
            font.add_text(
                &TextRenderConfig::new(
                    &[(&value, glm::vec4(20, 45, 90, 255))],
                    position,
                    0.5,
                )
                .set_depth(DISPLAY_DEPTH)
                .set_recenter(camera)
                .set_max_line_width(0.5)
                .set_align(TextAlign::Center),
            );
        }
    }
}
