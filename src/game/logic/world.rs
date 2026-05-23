
use std::array;
use nalgebra_glm as glm;

use crate::{MainContext, game::logic::{fabricator::Fabricator, robot::Robot}, open_simplex::{FractalSample, OpenSimplex}, stable_index_vector::StableIndexVector};

pub const CHUNK_MAG: usize = 8;
pub const CHUNK_SIZE: usize = 1 << CHUNK_MAG;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridPosition {
    None,
    Resource(u8),
    Obstacle,
}

impl Default for GridPosition {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug)]
pub struct Chunk {
    grid: [[GridPosition; CHUNK_SIZE]; CHUNK_SIZE],
    pending_updates: Vec<glm::U8Vec2>,
    pub fabricators: Vec<Fabricator>,
    pub robots: StableIndexVector<Robot>,
}

pub struct MinableGrid<'a> {
    grid: &'a mut [[GridPosition; CHUNK_SIZE]; CHUNK_SIZE],
    pending_updates: &'a mut Vec<glm::U8Vec2>,
}

impl<'a> MinableGrid<'a> {
    pub fn try_mine(&mut self, chunk_position: glm::U8Vec2) -> bool {
        let position = &mut self.grid[chunk_position.y as usize][chunk_position.x as usize];
        return match position {
            GridPosition::Resource(quantity) => {
                if *quantity == 1 {
                    *position = GridPosition::None;
                } else {
                    *quantity -= 1;
                }
                self.pending_updates.push(chunk_position);
                return true;
            },
            _ => false,
        };
    }

    pub fn try_release(&mut self, chunk_position: glm::U8Vec2) -> bool {
        let position = &mut self.grid[chunk_position.y as usize][chunk_position.x as usize];
        match position {
            GridPosition::Resource(quantity) => {
                if *quantity == u8::MAX {
                    false
                } else {
                    *quantity += 1;
                    self.pending_updates.push(chunk_position);
                    true
                }
            },
            GridPosition::None => {
                *position = GridPosition::Resource(1);
                self.pending_updates.push(chunk_position);
                true
            }
            _ => false,
        }
    }

    pub fn get(&self, chunk_position: glm::U8Vec2) -> GridPosition {
        self.grid[chunk_position.y as usize][chunk_position.x as usize]
    }
}

impl Chunk {
    pub fn new(fabricators: Vec<Fabricator>, robots: StableIndexVector<Robot>) -> Self {
        Self {
            grid: [[GridPosition::None; _]; _],
            pending_updates: vec![],
            fabricators, robots,
        }
    }

    pub fn new_proc_generated(chunk_id: glm::IVec2, noise: &OpenSimplex) -> Self {
        let start_position = chunk_id.map(|x| x as f32) * CHUNK_SIZE as f32;
        let grid = array::from_fn(|line| array::from_fn(|column| {
            let current_position = start_position + glm::vec2(column as f32, line as f32);
            let value = noise.fractal_sample(current_position, &FractalSample::new_harmonic::<6>(255.0, 0.05, 0.5, 2.0));
            if value <= 0.0 {
                GridPosition::None
            } else {
                GridPosition::Resource((value as u8).max(1))
            }
        }));

        return Self {
            grid,
            pending_updates: vec![],
            fabricators: vec![],
            robots: StableIndexVector::new(),
        };
    }

    pub fn get_pending_updates(&self) -> &[glm::U8Vec2] {
        &self.pending_updates
    }

    pub fn get_local_position(&self, position: glm::U8Vec2) -> GridPosition {
        return self.grid[position.y as usize][position.x as usize];
    }

    pub fn get_grid(&self) -> &[[GridPosition; CHUNK_SIZE]; CHUNK_SIZE] {
        &self.grid
    }

    pub fn get_grid_mut(&mut self) -> &mut [[GridPosition; CHUNK_SIZE]; CHUNK_SIZE] {
        &mut self.grid
    }

    pub fn update(&mut self, context: &mut MainContext) {
        self.pending_updates.clear();

        let mut minable_grid = MinableGrid {
            grid: &mut self.grid,
            pending_updates: &mut self.pending_updates,
        };

        for robot in self.robots.iter_unordered_mut() {
            robot.update(&mut minable_grid, context);
        }

        for fabricator in self.fabricators.iter_mut() {
            fabricator.update(context, &mut minable_grid, &mut self.robots);
        }

        self.handle_collisions();
    }
}
