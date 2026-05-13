use nalgebra_glm as glm;
use crate::{MainContext, game::logic::{computer::{MEMORY_SIZE, X0, X2}, fabricator::Fabricator, instruction::Instruction, memory::{InstructionMemory, RegisterMemory}, robot::Robot, world::World}, stable_index_vector::StableIndexVector};
use crate::game::logic::fabricator::Direction;

pub mod world;
pub mod compute_context;
pub mod computer;
pub mod display_module;
pub mod instruction;
pub mod memory;
pub mod mining_module;
pub mod movement_module;
pub mod robot;
pub mod fabricator;
pub mod collision;

const INSTRUCTION_PERIOD: f32 = 0.25;

pub struct GameState {
    world: World,
    robots: StableIndexVector<Robot>,
    pub fabricators: Vec<Fabricator>,
    pub robot_info_opened: Option<usize>,
    last_instruction_time_passed: f32,
}

impl GameState {
    pub fn new() -> Self {
        let mut robots = StableIndexVector::new();
        robots.push(Robot::new(glm::vec2(0.0, 0.0), [
            Instruction::new_lui(X2, 5),
            Instruction::new_sys(instruction::Syscall::MoveLeft),
            Instruction::new_subi(X2, X2, 1),
            Instruction::new_bne(X2, X0, 13),
            Instruction::new_sys(instruction::Syscall::MoveTop),
            Instruction::new_beq(X0, X0, 14),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        ]));

        Self {
            last_instruction_time_passed: 0.0,
            world: World::new(),
            fabricators: vec![Fabricator::new(glm::vec2(-5, 5), Direction::Left, Direction::Right)],
            robot_info_opened: None,
            robots,
        }
    }

    pub fn update(&mut self, context: &mut MainContext) {
        let mut is_instruction_tick = false;
        let delta = context.ui.get_delta();

        self.last_instruction_time_passed += delta;
        if self.last_instruction_time_passed >= INSTRUCTION_PERIOD {
            is_instruction_tick = true;
            self.last_instruction_time_passed -= INSTRUCTION_PERIOD;
        }

        for robot in self.robots.iter_unordered_mut() {
            robot.update(is_instruction_tick, &mut self.world, context);
        }

        for fabricator in self.fabricators.iter_mut() {
            fabricator.update(context, &mut self.world, &mut self.robots);
        }

        self.handle_collisions();
    }

    pub fn write_code(&mut self, robot_id: usize, code: [Instruction; MEMORY_SIZE]) {
        if let Some(robot) = self.robots.get_mut(robot_id) {
            robot.computer.instructions = InstructionMemory::new(code);
            robot.computer.registers = RegisterMemory::new([0; MEMORY_SIZE]);
        }
    }

    pub fn get_robots(&self) -> &StableIndexVector<Robot> {
        &self.robots
    }

    pub fn get_world(&self) -> &World {
        &self.world
    }
}
