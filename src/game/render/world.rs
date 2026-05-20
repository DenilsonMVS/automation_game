use std::{array, collections::HashMap};
use macros::{GlVertex, program_interface};
use nalgebra_glm as glm;

use crate::{game::logic::{GameState, world::{CHUNK_MAG, CHUNK_SIZE, Chunk, GridPosition}}, graphics::renderer::{Renderer, buffer::DynamicUnitBuffer, drawable::DrawMode, program::{Program, ShaderType}, uniform::UniformValue, vertex_array_object::{FieldType, StaticVertexLayout, VertexArrayObject}}};

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
            Self::Resource(quantity) => ChunkPosition {
                size: {
                    let max_qtd = 255;
                    ((*quantity as f32 / max_qtd as f32).sqrt() * 255.0).clamp(0.0, 255.0) as u8
                },
                id: 1,
            },
            Self::Obstacle => ChunkPosition { id: 2, size: 255 },
        }
    }
}

#[derive(Debug)]
pub struct ViewChunk<'a> {
    vao: VertexArrayObject<'a>,
    vbo: DynamicUnitBuffer<'a, ChunkPosition>,
}

impl<'a> ViewChunk<'a> {
    fn new(renderer: &'a Renderer, grid: &[[GridPosition; CHUNK_SIZE]; CHUNK_SIZE]) -> Self {
        let buffer_data: [_; CHUNK_SIZE * CHUNK_SIZE] = array::from_fn(|idx| {
            let i = idx / CHUNK_SIZE;
            let j = idx % CHUNK_SIZE;
            return grid[i][j].get_render_info();
        });

        let vbo = DynamicUnitBuffer::new(renderer, &buffer_data);
        let vao = VertexArrayObject::new(renderer, &[&vbo]);
        return Self { vao, vbo };
    }

    fn send_updates(&mut self, logic_chunk: &Chunk) {
        for update_pos in logic_chunk.get_pending_updates() {
            let index = update_pos.y as usize * CHUNK_SIZE + update_pos.x as usize;
            self.vbo.write(index, &logic_chunk.get_local_position(*update_pos).get_render_info());
        }
    }

    fn draw(&self) {
        self.vao.draw_instanced(4, (CHUNK_SIZE * CHUNK_SIZE) as i32, DrawMode::TriangleFan);
    }
}

#[program_interface(
	vert = "../../../res/shaders/chunk.vert",
	frag = "../../../res/shaders/simple_color.frag"
)]
struct ChunkProgram {
    u_vp: glm::Mat4,
    u_cunk_mag: i32,
    u_chunk_start_pos: glm::Vec2,
}

pub struct ViewWorld<'a> {
    chunks: HashMap<glm::IVec2, ViewChunk<'a>>,
    chunk_program: ChunkProgram<'a>,
}

impl<'a> ViewWorld<'a> {
    pub fn new(renderer: &'a Renderer, logic_state: &GameState) -> Self {
        let chunks: HashMap<glm::IVec2, ViewChunk> = logic_state.chunks
            .iter()
            .map(|(position, chunk)|
                (*position, ViewChunk::new(renderer, chunk.get_grid())))
            .collect();

        let chunk_program = ChunkProgram::init(renderer).unwrap();
        chunk_program.set_u_cunk_mag(&(CHUNK_MAG as i32));

        return Self { chunks, chunk_program };
    }

    pub fn send(&mut self, logic_state: &GameState) {
        for (position, logic_chunk) in logic_state.chunks.iter() {
            if let Some(view_chunk) = self.chunks.get_mut(position) {
                view_chunk.send_updates(logic_chunk);
            }
        }
    }

    pub fn draw(&self, proj_matrix: &glm::Mat4) {
        self.chunk_program.bind();
        self.chunk_program.set_u_vp(proj_matrix);
        for (id, chunk) in self.chunks.iter() {
            self.chunk_program.set_u_chunk_start_pos(&id.map(|x| (x * CHUNK_SIZE as i32) as f32));
            chunk.draw();
        }
    }
}

