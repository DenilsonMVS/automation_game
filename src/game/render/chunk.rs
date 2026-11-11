use std::array;

use macros::GlVertex;
use crate::{game::logic::top_level::chunk::{CHUNK_SIZE, Chunk, GridPosition}, graphics::renderer::{buffer::dynamic_buffer::DynamicBuffer, draw_mode::DrawMode, vertex_array_object::{FieldType, StaticVertexLayout, VertexArrayObject}}};
use nalgebra_glm as glm;

#[derive(Debug, Clone, Copy, GlVertex)]
#[vertex(divisor = 1)]
struct ChunkPosition {
    id: u8,
    #[normalized]
    size: u8,
}

impl GridPosition {
    fn get_render_info(&self) -> ChunkPosition {
        match self {
            Self::None => ChunkPosition { id: 0, size: 0 },
            Self::Resource(amount) => ChunkPosition {
                size: {
                    let qtd_norm = (amount.get() as f32) / 255.0;
                    let side_proportion = qtd_norm.sqrt();
                    (side_proportion * 255.0).round().clamp(0.0, 255.0) as u8
                },
                id: 1,
            },
            Self::Obstacle => ChunkPosition { id: 2, size: 255 },
        }
    }
}

#[derive(Debug)]
pub struct ViewChunk {
    vao: VertexArrayObject,
    vbo: DynamicBuffer<ChunkPosition>,
}

impl ViewChunk {
    pub fn new(logic_chunk: &Chunk) -> Self {
        let buffer_data: [_; CHUNK_SIZE * CHUNK_SIZE] = array::from_fn(|idx| {
            let i = idx / CHUNK_SIZE;
            let j = idx % CHUNK_SIZE;
            logic_chunk.grid[i][j].get_render_info()
        });

        let vbo = DynamicBuffer::new_initialized(&buffer_data);
        let vao = VertexArrayObject::new(&[&vbo]);
        return Self { vao, vbo };
    }

    pub fn update(&mut self, position: glm::IVec2, value: GridPosition) {
        let position = position.y * CHUNK_SIZE as i32 + position.x;
        self.vbo.write(&value.get_render_info(), position as u32);
    }

    pub fn draw(&self) {
        self.vao.draw_instanced(4, self.vbo.get_count() as i32, DrawMode::TriangleStrip);
    }
}