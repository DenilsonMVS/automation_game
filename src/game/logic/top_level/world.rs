use std::{collections::HashMap, num::NonZeroU8, sync::mpsc};
use nalgebra_glm as glm;

use crate::game::{logic::top_level::chunk::{CHUNK_SIZE, Chunk, GridPosition}, render::RenderMessage};

pub struct World {
    chunks: HashMap<glm::IVec2, Chunk>,
    updates: Vec<(glm::IVec2, GridPosition)>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            updates: vec![],
        }
    }

    pub fn add_chunk(&mut self, position: glm::IVec2, chunk: Chunk, render_tx: &mpsc::Sender<RenderMessage>) {
        render_tx.send(RenderMessage::ChunkGenerated((position, Box::new(chunk.clone())))).unwrap();
        self.chunks.insert(position, chunk);
    }

    pub fn get_amount(&self, position: glm::IVec2) -> u8 {
        let chunk_id = position.map(|x| x.div_euclid(CHUNK_SIZE as i32));
        let chunk_pos = position.map(|x| x.rem_euclid(CHUNK_SIZE as i32));
        let Some(chunk) = self.chunks.get(&chunk_id) else { return 0 };
        chunk.grid[chunk_pos.y as usize][chunk_pos.x as usize].get_amount()
    }

    pub fn take_updates(&mut self) -> Vec<(glm::IVec2, GridPosition)> {
        std::mem::take(&mut self.updates)
    }

    pub fn try_mine(&mut self, global_position: glm::IVec2) -> bool {
        let chunk_id = global_position.map(|x| x.div_euclid(CHUNK_SIZE as i32));
        let chunk_pos = global_position.map(|x| x.rem_euclid(CHUNK_SIZE as i32));

        let Some(chunk) = self.chunks.get_mut(&chunk_id) else { return false };
        let position = &mut chunk.grid[chunk_pos.y as usize][chunk_pos.x as usize];

        match *position {
            GridPosition::Resource(amount) => {
                *position = match NonZeroU8::new(amount.get() - 1) {
                    Some(remaining) => GridPosition::Resource(remaining),
                    None => GridPosition::None,
                };
                self.updates.push((global_position, *position));
                true
            },
            _ => false,
        }
    }
}
