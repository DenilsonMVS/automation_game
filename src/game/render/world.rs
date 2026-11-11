use std::collections::HashMap;

use crate::{game::{logic::top_level::{self, chunk::{CHUNK_MAG, CHUNK_SIZE, GridPosition}}, render::chunk::ViewChunk}, graphics::renderer::{Capability, Renderer, program::{Program, ShaderType, uniform::UniformValue}}};
use macros::program_interface;
use nalgebra_glm as glm;

#[program_interface(
	vert = "../../../res/shaders/chunk.vert",
	frag = "../../../res/shaders/filled.frag"
)]
struct ChunkProgram {
    u_vp: glm::Mat4,
    u_cunk_mag: i32,
    u_chunk_start_pos: glm::Vec2,
}

pub struct World {
    chunks: HashMap<glm::IVec2, ViewChunk>,
    chunk_program: ChunkProgram,
}

impl World {
    pub fn new() -> Self {
        let chunk_program = ChunkProgram::init().unwrap();
        chunk_program.set_u_cunk_mag(&(CHUNK_MAG as i32));
        
        Self {
            chunks: HashMap::new(),
            chunk_program,
        }
    }

    pub fn modify_chunk(&mut self, modifications: &[(glm::IVec2, GridPosition)]) {
        for (global_position, value) in modifications.iter() {
            let chunk_id = global_position.map(|x| x.div_euclid(CHUNK_SIZE as i32));
            let chunk_pos = global_position.map(|x| x.rem_euclid(CHUNK_SIZE as i32));
            if let Some(chunk) = self.chunks.get_mut(&chunk_id) {
                chunk.update(chunk_pos, *value);
            }
        }
    }

    pub fn add_chunk(&mut self, position: glm::IVec2, chunk: &top_level::chunk::Chunk) {
        self.chunks.insert(position, ViewChunk::new(chunk));
    }

    pub fn draw(&self, renderer: &mut Renderer, u_vp: &glm::Mat4) {
        renderer.disable(Capability::DepthTest);
        renderer.enable(Capability::Multisample);

        self.chunk_program.bind();
        self.chunk_program.set_u_vp(u_vp);

        for (pos, chunk) in self.chunks.iter() {
            self.chunk_program.set_u_chunk_start_pos(&pos.map(|x| x as f32));
            chunk.draw();
        }
    }
}