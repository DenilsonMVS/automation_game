
use std::{array, collections::HashMap};
use nalgebra_glm as glm;

use crate::open_simplex::{FractalSample, OpenSimplex};

pub const CHUNK_MAG: usize = 8;
pub const CHUNK_SIZE: usize = 1 << CHUNK_MAG;

#[derive(Debug, Clone, Copy)]
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
}

impl Chunk {
    fn new(chunk_id: glm::IVec2, noise: &OpenSimplex) -> Self {
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
        };
    }

    pub fn get_pending_updates(&self) -> &[glm::U8Vec2] {
        &self.pending_updates
    }

    fn get_position(&self, global_position: glm::IVec2) -> GridPosition {
        let chunk_pos = global_position.map(|x| x.rem_euclid(CHUNK_SIZE as i32) as usize);
        return self.grid[chunk_pos.y][chunk_pos.x];
    }

    fn get_position_mut(&mut self, global_position: glm::IVec2) -> &mut GridPosition {
        let chunk_pos = global_position.map(|x| x.rem_euclid(CHUNK_SIZE as i32) as usize);
        return &mut self.grid[chunk_pos.y][chunk_pos.x];
    }

    pub fn get_local_position(&self, position: glm::U8Vec2) -> GridPosition {
        return self.grid[position.y as usize][position.x as usize];
    }

    pub fn get_grid(&self) -> &[[GridPosition; CHUNK_SIZE]; CHUNK_SIZE] {
        &self.grid
    }
}

pub struct World {
    chunks: HashMap<glm::I32Vec2, Chunk>,
}

impl World {
    pub fn new() -> Self {
        let noise = OpenSimplex::new(0);
        let positions = [
            glm::vec2( 0,  0),
            glm::vec2(-1,  0),
            glm::vec2( 0, -1),
            glm::vec2(-1, -1),
        ];

        let mut chunks = HashMap::new();
        for position in positions {
            chunks.insert(
                position,
                Chunk::new(position, &noise),
            );
        }

        return Self { chunks };
    }

    pub fn get_chunks(&self) -> &HashMap<glm::I32Vec2, Chunk> {
        &self.chunks
    }

    pub fn get_position(&self, global_position: glm::IVec2) -> Option<GridPosition> {
        let chunk = self.get_chunk(global_position)?;
        return Some(chunk.get_position(global_position));
    }

    pub fn get_chunk(&self, global_position: glm::IVec2) -> Option<&Chunk> {
        let chunk_id = global_position.map(|x| x.div_euclid(CHUNK_SIZE as i32));
        return self.chunks.get(&chunk_id);
    }

    fn get_chunk_mut(&mut self, global_position: glm::IVec2) -> Option<&mut Chunk> {
        let chunk_id = global_position.map(|x| x.div_euclid(CHUNK_SIZE as i32));
        return self.chunks.get_mut(&chunk_id);
    }

    pub fn try_mine(&mut self, global_position: glm::IVec2) -> bool {
        let Some(chunk) = self.get_chunk_mut(global_position) else { return false };
        let position = chunk.get_position_mut(global_position);
        return match position {
            GridPosition::Resource(quantity) => {
                if *quantity == 1 {
                    *position = GridPosition::None;
                } else {
                    *quantity -= 1;
                }
                chunk.pending_updates.push(global_position.map(|x| x.rem_euclid(CHUNK_SIZE as i32) as u8));
                return true;
            },
            _ => false,
        };
    }

    pub fn try_release(&mut self, global_position: glm::IVec2) -> bool {
        let Some(chunk) = self.get_chunk_mut(global_position) else { return false };
        let position = chunk.get_position_mut(global_position);
        match position {
            GridPosition::Resource(quantity) => {
                if *quantity == u8::MAX {
                    false
                } else {
                    *quantity += 1;
                    chunk.pending_updates.push(global_position.map(|x| x.rem_euclid(CHUNK_SIZE as i32) as u8));
                    true
                }
            },
            GridPosition::None => {
                *position = GridPosition::Resource(1);
                chunk.pending_updates.push(global_position.map(|x| x.rem_euclid(CHUNK_SIZE as i32) as u8));
                true
            }
            _ => false,
        }
    }
}
