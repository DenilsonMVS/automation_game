use std::collections::HashMap;
use nalgebra_glm as glm;
use crate::{MainContext, game::logic::{compute_context::ComputeContext, computer::MEMORY_SIZE, fabricator::Fabricator, instruction::Instruction, memory::{InstructionMemory, RegisterMemory}, robot::Robot, world::Chunk}};


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
    pub chunks: HashMap<glm::I32Vec2, Chunk>,
}

impl GameState {
    pub fn new(chunks: HashMap<glm::I32Vec2, Chunk>) -> Self {
        Self { chunks }
    }

    pub fn update(&mut self, context: &mut MainContext) {
        for (_, chunk) in self.chunks.iter_mut() {
            chunk.update(context);
        }
    }

    pub fn write_code(&mut self, chunk_id: glm::IVec2, robot_id: u32, code: [Instruction; MEMORY_SIZE]) {
        if let Some(robot) = self.get_robot_mut(chunk_id, robot_id) {
            robot.computer.instructions = InstructionMemory::new(code);
            robot.computer.registers = RegisterMemory::new([0; MEMORY_SIZE]);
            robot.context = ComputeContext::Computer;
        }
    }

    pub fn get_robot(&self, chunk_id: glm::IVec2, robot_id: u32) -> Option<&Robot> {
        self.chunks.get(&chunk_id).and_then(|chunk| chunk.robots.get(robot_id as usize))
    }

    pub fn get_robot_mut(&mut self, chunk_id: glm::IVec2, robot_id: u32) -> Option<&mut Robot> {
        self.chunks.get_mut(&chunk_id).and_then(|chunk| chunk.robots.get_mut(robot_id as usize))
    }

    pub fn get_fabricator(&self, chunk_id: glm::IVec2, fabricator_idx: u32) -> Option<&Fabricator> {
        self.chunks.get(&chunk_id).and_then(|chunk| chunk.fabricators.get(fabricator_idx as usize))
    }

    pub fn get_fabricator_mut(&mut self, chunk_id: glm::IVec2, fabricator_idx: u32) -> Option<&mut Fabricator> {
        self.chunks.get_mut(&chunk_id).and_then(|chunk| chunk.fabricators.get_mut(fabricator_idx as usize))
    }
}
