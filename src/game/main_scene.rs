use nalgebra_glm as glm;
use crate::{game::{instruction::{Instruction, Syscall}, robot::Robot, scene::Scene, world::World}, graphics::renderer::Renderer};
use super::computer::X;

const INSTRUCTION_PERIOD: f32 = 0.05;

pub struct MainScene<'a> {
    last_instruction_time_passed: f32,
    world: World<'a>,
    robots: Vec<Robot>,
}

impl<'a> MainScene<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        Self {
            last_instruction_time_passed: 0.0,
            world: World::new(renderer),
            robots: vec![
                Robot::new(glm::vec2(0.0, 0.0))
                    .set_instruction(0, Instruction::new_sys(Syscall::Mine))
                    .set_instruction(1, Instruction::new_addi(X[15], X[2], 0))
                    .set_instruction(2, Instruction::new_sys(Syscall::Stored))
                    .set_instruction(3, Instruction::new_lui(X[14], 1))
                    .set_instruction(4, Instruction::new_sys(Syscall::Display))
                    .set_instruction(5, Instruction::new_bne(X[2], X[0], 10))
                    .set_instruction(6, Instruction::new_sys(Syscall::MoveRight))
                    .set_instruction(7, Instruction::new_beq(X[0], X[0], 8))
            ],
        }
    }
}

impl<'a> Scene for MainScene<'a> {    
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
            robot.update(context.delta, is_instruction_tick, &mut self.world);
        }
    }

    fn send(&mut self, context: &mut crate::MainContext) {
        let simple_shape_renderer = &mut context.simple_shape_renderer;
        let font = &mut context.font;

        for robot in self.robots.iter_mut() {
            robot.draw(simple_shape_renderer, font);
        }
    }

    fn generate_output(&mut self, context: &mut crate::MainContext) {
        self.world.draw(&context.vp_matrix);
    }

    fn handle_input(&mut self, context: &mut crate::MainContext) {
        
    }
}
