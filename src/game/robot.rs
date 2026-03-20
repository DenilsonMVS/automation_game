
use nalgebra_glm as glm;

use crate::{game::instruction::Instruction, graphics::renderer::{positioning::PositionMode, simple_shape_renderer::SimpleShapeRenderer}};

const MEMORY_RANK: usize = 4;
const MEMORY_SIZE: usize = 1 << MEMORY_RANK;
const PROGRAM_COUNTER_IDX: usize = 0;
const ROBOT_SIZE: f32 = 1.0;
const WEEL_DIAMETER: f32 = 0.3;
const WEEL_DISTANCE: f32 = 0.48;
const ROBOT_DEPTH: u16 = 5000;
const WEELS_DEPTH: u16 = ROBOT_DEPTH + 1;
const WEEL_COLOR: [u8; 3] = [45, 40, 40];

#[derive(Debug)]
pub struct Robot {
    memory: [u8; MEMORY_SIZE],
    position: glm::Vec2,
    speed: glm::Vec2,
    instructions: [Instruction; MEMORY_SIZE],
    color: glm::U8Vec3,
}

impl Robot {
    pub fn new() -> Self {
        Self {
            memory: [0; 16],
            position: glm::vec2(0.0, 0.0),
            speed: glm::vec2(0.0, 0.0),
            instructions: [Instruction::NoOp; MEMORY_SIZE],
            color: glm::vec3(200, 200, 190),
        }
    }

    pub fn draw(&self, simple_shape_renderer: &mut SimpleShapeRenderer) {
        let weel_offsets = [
            glm::vec2(-1.0, -1.0),
            glm::vec2( 1.0, -1.0),
            glm::vec2( 1.0,  1.0),
            glm::vec2(-1.0,  1.0),
        ];

        for weel_offset in weel_offsets {
            simple_shape_renderer.send_ellipse(
                self.position + weel_offset * WEEL_DISTANCE,
                glm::vec2(WEEL_DIAMETER, WEEL_DIAMETER),
                PositionMode::Center,
                glm::vec4(WEEL_COLOR[0], WEEL_COLOR[1], WEEL_COLOR[2], 255),
                WEELS_DEPTH,
            );
        }

        simple_shape_renderer.send_rectangle(
            self.position,
            glm::vec2(ROBOT_SIZE, ROBOT_SIZE),
            PositionMode::Center,
            glm::vec4(self.color.x, self.color.y, self.color.z, 255),
            ROBOT_DEPTH,
        );
    }
}