use std::array;

use nalgebra_glm as glm;
use crate::{game::{instruction::{Instruction, Syscall}, robot::Robot, scene::Scene}, graphics::renderer::{Renderer, positioning::PositionMode, simple_shape_renderer::SimpleShapeRenderer}, open_simplex::{FractalSample, OpenSimplex}};

const INSTRUCTION_PERIOD: f32 = 0.25;
const CHUNK_SIZE: usize = 1 << 8;

#[derive(Debug, Clone, Copy)]
enum GridPosition {
    None,
    Resource(u16, u8),
}

struct PositionRenderInfo {
    size: f32,
    color: glm::U8Vec4,
}

impl GridPosition {
    fn get_starting_amount(original_base_size: u8) -> u16 {
        (original_base_size as u16) * 256 + 255
    }

    fn get_render_info(&self) -> Option<PositionRenderInfo> {
        match self {
            Self::None => None,
            Self::Resource(quantity, original_size_base) => Some(PositionRenderInfo {
                size: {
                    let max_qtd = Self::get_starting_amount(*original_size_base);
                    (*quantity as f32 / max_qtd as f32).sqrt()
                },
                color: glm::vec4(20, 150, 20, 255),
            }),
        }
    }
}

impl Default for GridPosition {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug)]
struct Chunk {
    grid: [[GridPosition; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            grid: [[GridPosition::default(); _]; _]
        }
    }
}

impl Chunk {
    fn new(chunk_id: glm::IVec2, noise: &OpenSimplex) -> Self {
        let start_position = chunk_id.map(|x| x as f32) * CHUNK_SIZE as f32;
        return Self {
            grid: array::from_fn(|line| array::from_fn(|column| {
                let current_position = start_position + glm::vec2(column as f32, line as f32);
                let value = noise.fractal_sample(current_position, &FractalSample::new_harmonic::<6>(255.0, 0.05, 0.5, 2.0));
                if value <= 0.0 {
                    GridPosition::None
                } else {
                    GridPosition::Resource(value as u16, 0)
                }
            }))
        };
    }

    fn send(&self, initial_offset: glm::Vec2, simple_shape_renderer: &mut SimpleShapeRenderer) {
        for (i, line) in self.grid.iter().enumerate() {
            for (j, element) in line.iter().enumerate() {
                if let Some(PositionRenderInfo { color, size }) =
                    element.get_render_info()
                {
                    let offset = glm::vec2(j as f32, i as f32);
                    simple_shape_renderer.send_rectangle(
                        initial_offset + offset,
                        glm::vec2(size, size),
                        PositionMode::Center,
                        color,
                        40000, 0,
                    );
                }
            }
        }
    }

    fn update_position(&mut self, i: usize, j: usize, value: GridPosition) {
        self.grid[i][j] = value;
    }
}


pub struct MainScene {
    last_instruction_time_passed: f32,
    chunk: Chunk,
    robots: Vec<Robot>,
    // noise: OpenSimplex,
}

impl<'a> MainScene {
    pub fn new(_renderer: &'a Renderer) -> Self {
        let noise = OpenSimplex::new(0);
        let chunk = Chunk::new(glm::vec2(0, 0), &noise);

        return Self {
            last_instruction_time_passed: 0.0,
            // noise,
            chunk,
            robots: vec![
                Robot::new(glm::vec2(0.0, 0.0))
                    .set_instruction(0, Instruction::new_sys(Syscall::MoveRight))
                    .set_instruction(1, Instruction::new_sys(Syscall::MoveRight))
                    .set_instruction(2, Instruction::new_sys(Syscall::MoveRight))
            ],
        };
    }
}

impl Scene for MainScene {    
    fn game_logic(&mut self, context: &mut crate::MainContext) {
        if context.halt {
            return;
        }

        let mut is_instruction_tick = false;

        self.last_instruction_time_passed += context.delta;
        if self.last_instruction_time_passed >= INSTRUCTION_PERIOD {
            is_instruction_tick = true;
            self.last_instruction_time_passed -= INSTRUCTION_PERIOD;
        }

        for robot in self.robots.iter_mut() {
            robot.update(context.delta, is_instruction_tick);
        }
    }

    fn generate_output(&mut self, context: &mut crate::MainContext) {
        let simple_shape_renderer = &mut context.simple_shape_renderer;
        // let font = &mut context.font;

        for robot in self.robots.iter_mut() {
            robot.draw(simple_shape_renderer);
        }
        self.chunk.send(glm::vec2(0.0, 0.0), simple_shape_renderer);
    }

    fn handle_input(&mut self, context: &mut crate::MainContext) {
        
    }
}
