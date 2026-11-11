use crate::{game::{logic::display_module::DisplayModule, render::robot::ROBOT_DEPTH}, graphics::renderer::{Renderer, fonts::{TextAlign, TextRenderConfig, font::Font}}};
use nalgebra_glm as glm;

const DISPLAY_DEPTH: u16 = ROBOT_DEPTH - 1;

impl DisplayModule {
    pub fn send(&self, renderer: &mut Renderer, font: &mut Font, position: glm::Vec2) {
        if let Some(value) = self.get_value() {
            let value = value.to_string();
            renderer.font_renderer.push(
                &TextRenderConfig::new(
                    &[(&value, glm::vec4(20, 45, 90, 255))],
                    position,
                    0.5,
                )
                .with_depth(DISPLAY_DEPTH)
                .with_max_line_width(0.5)
                .with_align(TextAlign::Center),
                font,
                0,
            );
        }
    }
}
