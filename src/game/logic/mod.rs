use nalgebra_glm as glm;
use crate::{MainContext, game::logic::{computer::{PC, X0, X2, X3, X14, X15}, instruction::{Instruction, Syscall}, robot::Robot, world::World}};

pub mod world;
pub mod compute_context;
pub mod computer;
pub mod display_module;
pub mod instruction;
pub mod memory;
pub mod mining_module;
pub mod movement_module;
pub mod robot;

const INSTRUCTION_PERIOD: f32 = 0.25;

pub struct GameState {
    world: World,
    robots: Vec<Robot>,
    last_instruction_time_passed: f32,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            last_instruction_time_passed: 0.0,
            world: World::new(),
            robots: vec![
                Robot::new(glm::vec2(0.0, 0.0), [
                    Instruction::new_lui(X14, 1),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_sys(Syscall::Scan),
                    Instruction::new_addi(X15, X3, 0),
                    Instruction::new_beq(X3, X0, 6),
                    Instruction::new_sys(Syscall::Mine),
                    Instruction::new_sub(X3, X15, X3),
                    Instruction::new_add(X2, X15, X2),
                    Instruction::new_addi(X2, X15, 0),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_bne(X3, X0, 9),
                    Instruction::new_sys(Syscall::MoveRight),
                    Instruction::new_lui(PC, 1),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                ]),
                Robot::new(glm::vec2(0.0, -1.0), [
                    Instruction::new_lui(X14, 1),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_sys(Syscall::Scan),
                    Instruction::new_addi(X15, X3, 0),
                    Instruction::new_beq(X3, X0, 6),
                    Instruction::new_sys(Syscall::Mine),
                    Instruction::new_sub(X3, X15, X3),
                    Instruction::new_add(X2, X15, X2),
                    Instruction::new_addi(X2, X15, 0),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_bne(X3, X0, 9),
                    Instruction::new_sys(Syscall::MoveRight),
                    Instruction::new_lui(PC, 1),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                ]),
                Robot::new(glm::vec2(0.0, -2.0), [
                    Instruction::new_lui(X14, 1),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_sys(Syscall::Scan),
                    Instruction::new_addi(X15, X3, 0),
                    Instruction::new_beq(X3, X0, 6),
                    Instruction::new_sys(Syscall::Mine),
                    Instruction::new_sub(X3, X15, X3),
                    Instruction::new_add(X2, X15, X2),
                    Instruction::new_addi(X2, X15, 0),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_bne(X3, X0, 9),
                    Instruction::new_sys(Syscall::MoveRight),
                    Instruction::new_lui(PC, 1),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                ]),
                Robot::new(glm::vec2(0.0, 1.0), [
                    Instruction::new_lui(X14, 1),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_sys(Syscall::Scan),
                    Instruction::new_addi(X15, X3, 0),
                    Instruction::new_beq(X3, X0, 6),
                    Instruction::new_sys(Syscall::Mine),
                    Instruction::new_sub(X3, X15, X3),
                    Instruction::new_add(X2, X15, X2),
                    Instruction::new_addi(X2, X15, 0),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_bne(X3, X0, 9),
                    Instruction::new_sys(Syscall::MoveRight),
                    Instruction::new_lui(PC, 1),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                ]),
                Robot::new(glm::vec2(0.0, 2.0), [
                    Instruction::new_lui(X14, 1),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_sys(Syscall::Scan),
                    Instruction::new_addi(X15, X3, 0),
                    Instruction::new_beq(X3, X0, 6),
                    Instruction::new_sys(Syscall::Mine),
                    Instruction::new_sub(X3, X15, X3),
                    Instruction::new_add(X2, X15, X2),
                    Instruction::new_addi(X2, X15, 0),
                    Instruction::new_sys(Syscall::Display),
                    Instruction::new_bne(X3, X0, 9),
                    Instruction::new_sys(Syscall::MoveRight),
                    Instruction::new_lui(PC, 1),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                    Instruction::new_sys(Syscall::Nop),
                ]),
            ],
        }
    }

    pub fn update(&mut self, context: &mut MainContext) {
        let mut is_instruction_tick = false;
        let delta = context.input_state.get_delta();

        self.last_instruction_time_passed += delta;
        if self.last_instruction_time_passed >= INSTRUCTION_PERIOD {
            is_instruction_tick = true;
            self.last_instruction_time_passed -= INSTRUCTION_PERIOD;
        }

        for robot in self.robots.iter_mut() {
            robot.update(delta, is_instruction_tick, &mut self.world);
        }
    }

    pub fn get_robots(&self) -> &[Robot] {
        &self.robots
    }

    pub fn get_world(&self) -> &World {
        &self.world
    }
}
