use nalgebra_glm as glm;
use crate::{game::{instruction::{Instruction, Syscall}, robot::Robot, scene::Scene, world::World}, graphics::renderer::Renderer};
use super::computer::X;

const INSTRUCTION_PERIOD: f32 = 0.25;

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
                Robot::new(glm::vec2(0.0, 0.0), &[
                    Instruction::new_lui(X[14], 1),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_sys(Syscall::Scan),
                    Instruction::new_addi(X[15], X[3], 0),
                    Instruction::new_beq(X[3], X[0], 6),
                    Instruction::new_sys(Syscall::Mine),
                    Instruction::new_sub(X[3], X[15], X[3]),
                    Instruction::new_add(X[2], X[15], X[2]),
                    Instruction::new_addi(X[2], X[15], 0),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_bne(X[3], X[0], 9),
                    Instruction::new_sys(Syscall::MoveRight),
                    Instruction::new_lui(X[1], 1),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                ]),
                Robot::new(glm::vec2(0.0, -1.0), &[
                    Instruction::new_lui(X[14], 1),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_sys(Syscall::Scan),
                    Instruction::new_addi(X[15], X[3], 0),
                    Instruction::new_beq(X[3], X[0], 6),
                    Instruction::new_sys(Syscall::Mine),
                    Instruction::new_sub(X[3], X[15], X[3]),
                    Instruction::new_add(X[2], X[15], X[2]),
                    Instruction::new_addi(X[2], X[15], 0),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_bne(X[3], X[0], 9),
                    Instruction::new_sys(Syscall::MoveRight),
                    Instruction::new_lui(X[1], 1),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                ]),
                Robot::new(glm::vec2(0.0, -2.0), &[
                    Instruction::new_lui(X[14], 1),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_sys(Syscall::Scan),
                    Instruction::new_addi(X[15], X[3], 0),
                    Instruction::new_beq(X[3], X[0], 6),
                    Instruction::new_sys(Syscall::Mine),
                    Instruction::new_sub(X[3], X[15], X[3]),
                    Instruction::new_add(X[2], X[15], X[2]),
                    Instruction::new_addi(X[2], X[15], 0),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_bne(X[3], X[0], 9),
                    Instruction::new_sys(Syscall::MoveRight),
                    Instruction::new_lui(X[1], 1),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                ]),
                Robot::new(glm::vec2(0.0, 1.0), &[
                    Instruction::new_lui(X[14], 1),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_sys(Syscall::Scan),
                    Instruction::new_addi(X[15], X[3], 0),
                    Instruction::new_beq(X[3], X[0], 6),
                    Instruction::new_sys(Syscall::Mine),
                    Instruction::new_sub(X[3], X[15], X[3]),
                    Instruction::new_add(X[2], X[15], X[2]),
                    Instruction::new_addi(X[2], X[15], 0),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_bne(X[3], X[0], 9),
                    Instruction::new_sys(Syscall::MoveRight),
                    Instruction::new_lui(X[1], 1),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                ]),
                Robot::new(glm::vec2(0.0, 2.0), &[
                    Instruction::new_lui(X[14], 1),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_sys(Syscall::Scan),
                    Instruction::new_addi(X[15], X[3], 0),
                    Instruction::new_beq(X[3], X[0], 6),
                    Instruction::new_sys(Syscall::Mine),
                    Instruction::new_sub(X[3], X[15], X[3]),
                    Instruction::new_add(X[2], X[15], X[2]),
                    Instruction::new_addi(X[2], X[15], 0),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_bne(X[3], X[0], 9),
                    Instruction::new_sys(Syscall::MoveRight),
                    Instruction::new_lui(X[1], 1),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                ]),
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
            robot.draw(simple_shape_renderer, font, context.camera);
        }

        self.world.send_updates();
    }

    fn generate_output(&mut self, context: &mut crate::MainContext) {
        self.world.draw(&context.vp_matrix);
    }

    fn handle_input(&mut self, context: &mut crate::MainContext) {
        
    }
}
