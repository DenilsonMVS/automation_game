use crate::{MainContext, game::logic::{compute_context::ComputeContext, computer::MEMORY_SIZE, fabricator::Fabricator, instruction::Instruction, memory::{InstructionMemory, RegisterMemory}, robot::Robot, world::World}, stable_index_vector::StableIndexVector};


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

#[derive(Debug)]
pub struct GameState {
    world: World,
    robots: StableIndexVector<Robot>,
    pub fabricators: Vec<Fabricator>,
    pub robot_info_opened: Option<usize>,
}

impl GameState {
    pub fn new(world: World, fabricators: Vec<Fabricator>, robots: StableIndexVector<Robot>) -> Self {
        Self {
            world,
            fabricators,
            robot_info_opened: None,
            robots,
        }
    }

    pub fn update(&mut self, context: &mut MainContext) {
        for robot in self.robots.iter_unordered_mut() {
            robot.update(&mut self.world, context);
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
            robot.context = ComputeContext::Computer;
        }
    }

    pub fn get_robots(&self) -> &StableIndexVector<Robot> {
        &self.robots
    }

    pub fn get_world(&self) -> &World {
        &self.world
    }
}
