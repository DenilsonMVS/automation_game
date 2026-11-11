
use std::{array, num::NonZeroU8};

use nalgebra_glm as glm;

use crate::open_simplex::{FractalSample, OpenSimplex};

pub const CHUNK_MAG: usize = 8;
pub const CHUNK_SIZE: usize = 1 << CHUNK_MAG;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridPosition {
    None,
    Resource(NonZeroU8),
    Obstacle,
}

impl GridPosition {
    pub fn get_amount(&self) -> u8 {
        match self {
            Self::Resource(amount) => amount.get(),
            _ => 0,
        }
    }
}

impl Default for GridPosition {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub grid: [[GridPosition; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
    pub fn new(grid: [[GridPosition; CHUNK_SIZE]; CHUNK_SIZE]) -> Self {
        Self {
            grid,
        }
    }

    pub fn proc_generated(chunk_id: glm::IVec2, noise: &OpenSimplex) -> [[GridPosition; CHUNK_SIZE]; CHUNK_SIZE] {
        let start_position = chunk_id.map(|x| x as f32) * CHUNK_SIZE as f32;
        array::from_fn(|line| array::from_fn(|column| {
            let current_position = start_position + glm::vec2(column as f32, line as f32);
            let value = noise.fractal_sample(current_position, &FractalSample::new_harmonic::<6>(255.0, 0.05, 0.5, 2.0));
            if value <= 0.0 {
                GridPosition::None
            } else {
                let value_u8 = value.clamp(0.0, 255.0).round() as u8;
                if let Some(non_zero) = NonZeroU8::new(value_u8) {
                    GridPosition::Resource(non_zero)
                } else {
                    GridPosition::None
                }   
            }
        }))
    }
}
