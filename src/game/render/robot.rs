use nalgebra_glm as glm;
use crate::{game::logic::robot::Robot, graphics::{camera::Camera, renderer::{fonts::Font, positioning::PositionMode, simple_shape_renderer::SimpleShapeRenderer}}};

const ROBOT_SIZE: f32 = 0.8;
const WEEL_DIAMETER: f32 = 0.26;
const WEEL_DISTANCE: f32 = 0.37;
const ROBOT_DEPTH: u16 = 5000;
const WEELS_DEPTH: u16 = ROBOT_DEPTH + 1;
const WEEL_COLOR: [u8; 3] = [45, 40, 40];

impl Robot {
    pub fn draw_vec(&self, simple_shape_renderer: &mut SimpleShapeRenderer, vec: glm::Vec2, color: glm::U8Vec4) {
        let size = glm::length(&vec);
        simple_shape_renderer.send_rotated_rectangle(
            self.get_position(),
            glm::vec2(0.5, 0.04) * size,
            PositionMode::CenterLeft,
            glm::vec2(-vec.y, vec.x),
            color, 2000, 0);
        
        simple_shape_renderer.send_symmetrical_triangle(
            self.get_position() + vec * 0.6,
            self.get_position() + vec * 0.5,
            size * 0.1,
            color, 2000, 0);
    }

    pub fn send(&self, simple_shape_renderer: &mut SimpleShapeRenderer, font: &mut Font, camera: &Camera) {
        let weel_offsets = [
            glm::vec2(-1.0, -1.0),
            glm::vec2( 1.0, -1.0),
            glm::vec2( 1.0,  1.0),
            glm::vec2(-1.0,  1.0),
        ];

        for weel_offset in weel_offsets {
            simple_shape_renderer.send_ellipse(
                self.get_position()  + weel_offset * WEEL_DISTANCE,
                glm::vec2(WEEL_DIAMETER, WEEL_DIAMETER),
                PositionMode::Center,
                glm::vec4(WEEL_COLOR[0], WEEL_COLOR[1], WEEL_COLOR[2], 255),
                WEELS_DEPTH, 0
            );
        }

        simple_shape_renderer.send_rectangle(
            self.get_position(),
            glm::vec2(ROBOT_SIZE, ROBOT_SIZE),
            PositionMode::Center,
            self.get_color(),
            ROBOT_DEPTH, 0
        );

        self.get_display_module().send(font, self.get_position(), camera);
    }
}
