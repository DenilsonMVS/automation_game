
use std::{array, collections::HashMap};

use macros::{GlVertex, program_interface};
use nalgebra_glm as glm;

use crate::{graphics::renderer::{Renderer, buffer::DynamicUnitBuffer, drawable::DrawMode, program::{Program, ShaderType}, uniform::UniformValue, vertex_array_object::{FieldType, StaticVertexLayout, VertexArrayObject}}, open_simplex::{FractalSample, OpenSimplex}};

const CHUNK_MAG: usize = 8;
const CHUNK_SIZE: usize = 1 << CHUNK_MAG;

#[derive(Debug, Clone, Copy)]
pub enum GridPosition {
    None,
    Resource(u16, u8),
}

impl GridPosition {
    fn get_starting_amount(original_base_size: u8) -> u16 {
        (original_base_size as u16) * 256 + 255
    }

    fn get_render_info(&self) -> ChunkPosition {
        match self {
            Self::None => ChunkPosition { color: glm::vec4(0, 0,0, 0), size: 0.0 },
            Self::Resource(quantity, original_size_base) => ChunkPosition {
                size: {
                    let max_qtd = Self::get_starting_amount(*original_size_base);
                    (*quantity as f32 / max_qtd as f32).sqrt()
                },
                color: glm::vec4(20, 150, 20, 255),
            },
        }
    }
}

impl Default for GridPosition {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy, GlVertex)]
#[vertex(divisor = 1)]
struct ChunkPosition {
    #[normalized]
    color: glm::U8Vec4,
    size: f32,
}

pub trait GenericChunk {
    fn get_position(&self, global_position: glm::IVec2) -> GridPosition;
    fn get_position_mut(&mut self, global_position: glm::IVec2) -> &mut GridPosition;
}

#[derive(Debug)]
pub struct Chunk<'a> {
    grid: [[GridPosition; CHUNK_SIZE]; CHUNK_SIZE],
    vao: VertexArrayObject<'a>,
    vbo: DynamicUnitBuffer<'a, ChunkPosition>,
    pending_updates: Vec<glm::U8Vec2>,
}

impl<'a> Chunk<'a> {
    fn new(renderer: &'a Renderer, chunk_id: glm::IVec2, noise: &OpenSimplex) -> Self {
        let start_position = chunk_id.map(|x| x as f32) * CHUNK_SIZE as f32;
        let grid = array::from_fn(|line| array::from_fn(|column| {
            let current_position = start_position + glm::vec2(column as f32, line as f32);
            let value = noise.fractal_sample(current_position, &FractalSample::new_harmonic::<6>(255.0, 0.05, 0.5, 2.0));
            if value <= 0.0 {
                GridPosition::None
            } else {
                GridPosition::Resource((value as u16).max(1), 0)
            }
        }));

        let buffer_data: [_; CHUNK_SIZE * CHUNK_SIZE] = array::from_fn(|idx| {
            let i = idx / CHUNK_SIZE;
            let j = idx % CHUNK_SIZE;
            return grid[i][j].get_render_info();
        });

        let vbo = DynamicUnitBuffer::new(renderer, &buffer_data);
        let vao = VertexArrayObject::new(renderer, &[&vbo]);

        return Self {
            grid,
            vbo,
            vao,
            pending_updates: vec![],
        };
    }

    fn send_updates(&mut self) {
        while let Some(update_pos) = self.pending_updates.pop() {
            let update_pos = update_pos.map(|x| x as usize);
            let index = update_pos.y * CHUNK_SIZE + update_pos.x;
            self.vbo.write(index, self.grid[update_pos.y][update_pos.x].get_render_info());
        }
    }

    fn draw(&self) {
        self.vao.draw_instanced(4, (CHUNK_SIZE * CHUNK_SIZE) as i32, DrawMode::TriangleFan);
    }
}

impl<'a> GenericChunk for Chunk<'a> {
    fn get_position(&self, global_position: glm::IVec2) -> GridPosition {
        let chunk_pos = global_position.map(|x| x.rem_euclid(CHUNK_SIZE as i32) as usize);
        return self.grid[chunk_pos.y][chunk_pos.x];
    }

    fn get_position_mut(&mut self, global_position: glm::IVec2) -> &mut GridPosition {
        let chunk_pos = global_position.map(|x| x.rem_euclid(CHUNK_SIZE as i32) as usize);
        return &mut self.grid[chunk_pos.y][chunk_pos.x];
    }
}


#[program_interface(
	vert = "../../res/shaders/chunk.vert",
	frag = "../../res/shaders/simple_color.frag"
)]
struct ChunkProgram {
    u_vp: glm::Mat4,
    u_cunk_mag: i32,
    u_chunk_start_pos: glm::Vec2,
}

pub trait GenericWorld<T: GenericChunk> {
    fn get_position(&self, global_position: glm::IVec2) -> Option<GridPosition>;
    fn get_position_mut(&mut self, global_position: glm::IVec2) -> Option<&mut GridPosition>;
    fn get_chunk(&self, global_position: glm::IVec2) -> Option<&T>;
    fn get_chunk_mut(&mut self, global_position: glm::IVec2) -> Option<&mut T>;
    fn try_mine(&mut self, global_position: glm::IVec2) -> bool;
}

pub struct World<'a> {
    chunks: HashMap<glm::I32Vec2, Chunk<'a>>,
    chunk_program: ChunkProgram<'a>,
}

impl<'a> World<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
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
                Chunk::new(renderer, position, &noise),
            );
        }

        let chunk_program = ChunkProgram::init(renderer).unwrap();
        chunk_program.set_u_cunk_mag(&(CHUNK_MAG as i32));

        return Self { chunks, chunk_program };
    }

    pub fn send_updates(&mut self) {
        for (_, chunk) in self.chunks.iter_mut() {
            chunk.send_updates();
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

impl<'a> GenericWorld<Chunk<'a>> for World<'a> {
    fn get_position(&self, global_position: glm::IVec2) -> Option<GridPosition> {
        let chunk = self.get_chunk(global_position)?;
        return Some(chunk.get_position(global_position));
    }

    fn get_position_mut(&mut self, global_position: glm::IVec2) -> Option<&mut GridPosition> {
        let chunk = self.get_chunk_mut(global_position)?;
        return Some(chunk.get_position_mut(global_position));
    }

    fn get_chunk(&self, global_position: glm::IVec2) -> Option<&Chunk<'a>> {
        let chunk_id = global_position.map(|x| x.div_euclid(CHUNK_SIZE as i32));
        return self.chunks.get(&chunk_id);
    }

    fn get_chunk_mut(&mut self, global_position: glm::IVec2) -> Option<&mut Chunk<'a>> {
        let chunk_id = global_position.map(|x| x.div_euclid(CHUNK_SIZE as i32));
        return self.chunks.get_mut(&chunk_id);
    }

    fn try_mine(&mut self, global_position: glm::IVec2) -> bool {
        let Some(chunk) = self.get_chunk_mut(global_position) else { return false };
        let position = chunk.get_position_mut(global_position);
        return match position {
            GridPosition::None => false,
            GridPosition::Resource(quantity, _) => {
                if *quantity == 1 {
                    *position = GridPosition::None;
                } else {
                    *quantity -= 1;
                }
                chunk.pending_updates.push(global_position.map(|x| x.rem_euclid(CHUNK_SIZE as i32) as u8));
                return true;
            }
        };
    }
}



#[cfg(test)]
mod world_math_tests {
    use super::*;

    #[test]
    fn test_resource_area_scaling_logic() {
        let base_size = 1; 
        let max_qtd = GridPosition::get_starting_amount(base_size);
        
        let test_quantities = [
            max_qtd,
            max_qtd / 2,
            max_qtd / 4,
            1,
            0,
        ];

        for qty in test_quantities {
            let res = GridPosition::Resource(qty, base_size);
            let info = res.get_render_info();
            
            // A proporção real de área que o código DEVE retornar
            let expected_ratio = qty as f32 / max_qtd as f32;
            let actual_ratio = info.size * info.size;

            let diff = (actual_ratio - expected_ratio).abs();
            
            assert!(
                diff < 0.0001, 
                "Erro de precisão para qty {}: esperado {}, obtido {}", 
                qty, expected_ratio, actual_ratio
            );
        }
    }

    #[test]
    fn test_resource_visual_radius_sanity() {
        // Teste específico para garantir que a raiz quadrada está sendo aplicada
        // Se a quantidade é 1/4 (0.25), o 'size' (lado) deve ser 1/2 (0.5)
        let base_size = 0; // max_qtd = 255
        let qty = 64; // Aproximadamente 25% de 255
        
        let res = GridPosition::Resource(qty, base_size);
        let info = res.get_render_info();
        
        // sqrt(64/255) ≈ sqrt(0.25) ≈ 0.5
        assert!(info.size > 0.49 && info.size < 0.51, "O side/lado deveria ser ~0.5 para qty 64/255");
    }
}
