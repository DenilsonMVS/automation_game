use crate::{game::logic::{GameState, computer::MEMORY_SIZE, instruction::Instruction, robot::Robot, world::{Chunk, GridPosition, World}}, stable_index_vector::StableIndexVector};
use nalgebra_glm as glm;

pub fn create_first_level() -> GameState {
    let mut chunk = Chunk::new();
    let grid = chunk.get_grid_mut();

    grid[1][0] = GridPosition::Obstacle;
    grid[1][6] = GridPosition::Obstacle;
    for i in 0..7 {
        grid[0][i] = GridPosition::Obstacle;
        grid[2][i] = GridPosition::Obstacle;
    }

    let mut world = World::new();
    world.set_chunk(glm::vec2(0, 0), chunk);

    let mut robots = StableIndexVector::with_capacity(1);
    robots.push(Robot::new(glm::vec2(1.0, 1.0), [Instruction::default(); MEMORY_SIZE]));

    GameState::new(
        world,
        vec![],
        robots,
    )
}
