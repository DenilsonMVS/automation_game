
use nalgebra_glm as glm;

use crate::graphics::renderer::{positioning::PositionMode, simple_shape_renderer::SimpleShapeRenderer};

#[derive(Debug)]
pub struct Graph {
    center: glm::Vec2,
    dimensions: glm::Vec2,
    primary_color: glm::U8Vec4,
    secondary_color: glm::U8Vec4,
}

impl Graph {
    pub fn new(center: glm::Vec2, dimensions: glm::Vec2, primary_color: glm::U8Vec4, secondary_color: glm::U8Vec4) -> Self {
        Self { center, dimensions, primary_color, secondary_color }
    }

    pub fn draw(&self, simple_shape_renderer: &mut SimpleShapeRenderer) {
        let min_pos = self.center - self.dimensions * 0.5;
        simple_shape_renderer.send_rectangle(
            min_pos - glm::vec2(0.0025, 0.0025),
            glm::vec2(0.005, self.dimensions.y),
            PositionMode::BottomLeft,
            self.secondary_color,
            3000,
        );
        simple_shape_renderer.send_rectangle(
            min_pos - glm::vec2(0.0025, 0.0025),
            glm::vec2(self.dimensions.x, 0.005),
            PositionMode::BottomLeft,
            self.secondary_color,
            3000,
        );

        simple_shape_renderer.send_symmetrical_triangle(
            min_pos + glm::vec2(0.0, self.dimensions.y + 0.025),
            min_pos + glm::vec2(0.0, self.dimensions.y),
            0.025,
            self.secondary_color,
            3000
        );
        simple_shape_renderer.send_symmetrical_triangle(
            min_pos + glm::vec2(self.dimensions.x + 0.025, 0.0),
            min_pos + glm::vec2(self.dimensions.x, 0.0),
            0.025,
            self.secondary_color,
            3000
        );
    }
}