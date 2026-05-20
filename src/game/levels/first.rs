use std::collections::HashMap;

use crate::{game::{logic::{GameState, computer::MEMORY_SIZE, instruction::Instruction, robot::Robot, world::{Chunk, GridPosition}}, render::RenderState, scene::Scene}, graphics::renderer::Renderer, stable_index_vector::StableIndexVector};
use nalgebra_glm as glm;

fn create_first_level() -> GameState {
    let mut robots = StableIndexVector::with_capacity(1);
    robots.push(Robot::new(glm::vec2(1.0, 1.0), [Instruction::default(); MEMORY_SIZE]));
    robots.push(Robot::new(glm::vec2(3.0, 1.0), [Instruction::default(); MEMORY_SIZE]));

    let mut chunk = Chunk::new(vec![], robots);
    let grid = chunk.get_grid_mut();

    grid[1][0] = GridPosition::Obstacle;
    grid[1][6] = GridPosition::Obstacle;
    for i in 0..7 {
        grid[0][i] = GridPosition::Obstacle;
        grid[2][i] = GridPosition::Obstacle;
    }

    let mut chunks = HashMap::new();
    chunks.insert(glm::vec2(0, 0), chunk);

    GameState::new(chunks)
}

pub struct FirstLevel<'a> {
    pub logic: GameState,
    pub render: RenderState<'a>,
}

impl<'a> FirstLevel<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        let logic = create_first_level();
        let render = RenderState::new(renderer, &logic);
        Self {
            logic,
            render,
        }
    }
}

impl<'a> Scene for FirstLevel<'a> {
    fn game_logic(&mut self, context: &mut crate::MainContext) {
        self.logic.update(context);
        self.render.update(&mut self.logic, context, true);
    }

    fn send(&mut self, context: &mut crate::MainContext) {
        self.render.send(&self.logic, context);
    }

    fn generate_output(&mut self, context: &mut crate::MainContext) {
        self.render.draw(&context.ui.camera.get_vp_matrix(&context.ui.get_projection_matrix()));
    }
}