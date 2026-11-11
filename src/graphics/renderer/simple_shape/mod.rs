
use std::num::NonZeroU32;

use nalgebra_glm as glm;
use macros::{GlVertex, program_interface};

use crate::graphics::renderer::{buffer::{block_buffer::BlockBuffer, stream_buffer::StreamBuffer}, draw_mode::DrawMode, positioning::RenderBox, program::{Program, ShaderType, uniform::UniformValue}, vertex_array_object::{DrawArraysIndirectCommand, FieldType, StaticVertexLayout, VertexArrayObject}};
use anyhow::{Result};

#[program_interface(
	vert = "../../../../res/shaders/filled.vert",
	frag = "../../../../res/shaders/filled.frag"
)]
pub(crate) struct FilledProgram {
    u_vp: glm::Mat4,
}

#[program_interface(
	vert = "../../../../res/shaders/ellipse.vert",
	frag = "../../../../res/shaders/ellipse.frag"
)]
pub(crate) struct EllipseProgram {
    u_vp: glm::Mat4,
}

#[repr(C)]
#[derive(Debug, GlVertex)]
#[vertex(divisor = 1)]
pub struct SimpleShapeVertex {
    pub v1: glm::Vec2,
    pub v2: glm::Vec2,
    pub v3: glm::Vec2,
    #[normalized]
    pub color: glm::U8Vec4,
    #[normalized]
    pub depth: u16,
}

pub struct Layer {
    triangles_cursor: Option<u32>,
    rectangles_cursor: Option<u32>,
    ellipses_cursor: Option<u32>,
}

impl Layer {
    fn new() -> Self {
        Self {
            triangles_cursor: None,
            rectangles_cursor: None,
            ellipses_cursor: None,
        }
    }
}

pub struct SimpleShapeRenderer {
    command_buffer: StreamBuffer<DrawArraysIndirectCommand>,
    vbo: BlockBuffer<SimpleShapeVertex>,
    vao: VertexArrayObject,

    layers: Vec<Layer>,

    filled_program: FilledProgram,
    ellipse_program: EllipseProgram,
}

impl SimpleShapeRenderer {
    pub fn new() -> Result<Self> {
        const COMMAND_BUFFER_INITIAL_CAPACITY: NonZeroU32 = NonZeroU32::new(0x100).unwrap();
        
        let vbo = BlockBuffer::new();
        let vao = VertexArrayObject::new(&[&vbo]);
        let command_buffer = StreamBuffer::new(COMMAND_BUFFER_INITIAL_CAPACITY);

        let filled_program = FilledProgram::init()?;
        let ellipse_program = EllipseProgram::init()?;

        Ok(Self {
            command_buffer,
            vbo,
            vao,
            filled_program,
            ellipse_program,
            layers: vec![],
        })
    }

    // fn get_scissor_cut(bbox: &RenderBox, window_size: glm::Vec2, vp_matrix: &glm::Mat4) -> ScissorCut {
    //     let min = glm::vec4(bbox.min.x, bbox.min.y, 0.0, 1.0);
    //     let max = glm::vec4(bbox.max.x, bbox.max.y, 0.0, 1.0);
    //     let min_clip = vp_matrix * min;
    //     let max_clip = vp_matrix * max;

    //     let p1_ndc = glm::vec2(min_clip.x / min_clip.w, min_clip.y / min_clip.w);
    //     let p2_ndc = glm::vec2(max_clip.x / max_clip.w, max_clip.y / max_clip.w);

    //     let p1_screen = glm::vec2(
    //         (p1_ndc.x + 1.0) * 0.5 * window_size.x,
    //         (p1_ndc.y + 1.0) * 0.5 * window_size.y,
    //     );
    //     let p2_screen = glm::vec2(
    //         (p2_ndc.x + 1.0) * 0.5 * window_size.x,
    //         (p2_ndc.y + 1.0) * 0.5 * window_size.y,
    //     );

    //     let screen_x_min = p1_screen.x.min(p2_screen.x).round() as i32;
    //     let screen_x_max = p1_screen.x.max(p2_screen.x).round() as i32;
    //     let screen_y_min = p1_screen.y.min(p2_screen.y).round() as i32;
    //     let screen_y_max = p1_screen.y.max(p2_screen.y).round() as i32;

    //     let win_w = window_size.x as i32;
    //     let win_h = window_size.y as i32;

    //     let x = screen_x_min.clamp(0, win_w);
    //     let y = screen_y_min.clamp(0, win_h);
    //     let x2 = screen_x_max.clamp(0, win_w);
    //     let y2 = screen_y_max.clamp(0, win_h);

    //     let width = (x2 - x).max(0);
    //     let height = (y2 - y).max(0);

    //     ScissorCut {
    //         pos: glm::vec2(x, y),
    //         size: glm::vec2(width, height)
    //     }
    // }

    pub fn draw(&mut self, layer_idx: usize, vp_matrix: &glm::Mat4) {
        let Some(layer) = self.layers.get(layer_idx) else { return };

        let start_pos = self.command_buffer.get_cursor_pos();

        if let Some(triangle_cursor) = layer.triangles_cursor {
            for block_info in self.vbo.get_cursor_blocks(triangle_cursor) {
                self.command_buffer.push(&DrawArraysIndirectCommand {
                    count: 3,
                    instance_count: block_info.used_size,
                    first: 0,
                    base_instance: block_info.block_start,
                });
            }
        }

        if let Some(rectangles_cursor) = layer.rectangles_cursor {
            for block_info in self.vbo.get_cursor_blocks(rectangles_cursor) {
                self.command_buffer.push(&DrawArraysIndirectCommand {
                    count: 4,
                    instance_count: block_info.used_size,
                    first: 0,
                    base_instance: block_info.block_start,
                });
            }
        }

        let filled_pos = self.command_buffer.get_cursor_pos();

        if let Some(ellipses_cursor) = layer.ellipses_cursor {
            for block_info in self.vbo.get_cursor_blocks(ellipses_cursor) {
                self.command_buffer.push(&DrawArraysIndirectCommand {
                    count: 4,
                    instance_count: block_info.used_size,
                    first: 0,
                    base_instance: block_info.block_start,
                });
            }
        }

        if self.vbo.take_rebind() {
            self.vao.rebind(0, &self.vbo);
        }

        let ellipse_pos = self.command_buffer.get_cursor_pos();
        let filled_count = filled_pos - start_pos;
        let ellipse_count = ellipse_pos - filled_pos;

        if filled_count > 0 {
            self.filled_program.bind();
            self.filled_program.set_u_vp(vp_matrix);

            unsafe { gl::Enable(gl::MULTISAMPLE); }        
            self.vao.draw_multi_instanced(
                &self.command_buffer,
                start_pos,
                filled_count,
                DrawMode::TriangleStrip,
            );
        }

        if ellipse_count > 0 {
            self.ellipse_program.bind();
            self.ellipse_program.set_u_vp(vp_matrix);

            unsafe {
                gl::Disable(gl::MULTISAMPLE);
                gl::DepthMask(gl::FALSE);
            }
            
            self.vao.draw_multi_instanced(
                &self.command_buffer,
                filled_pos,
                ellipse_count,
                DrawMode::TriangleStrip,
            );

            unsafe {
                gl::DepthMask(gl::TRUE);
            }
        }
    }

    pub fn reset(&mut self) {
        self.layers.clear();
        self.vbo.reset();
        self.command_buffer.reset();
    }

    pub fn create_layer_if_doesnt_exist(&mut self, layer_idx: usize) {
        while self.layers.len() <= layer_idx {
            self.layers.push(Layer::new());
        }
    }

    pub fn send_render_box(
        &mut self,
        bbox: &RenderBox,
        color: glm::U8Vec4,
        depth: u16,
        layer_idx: usize,
    ) {
        self.create_layer_if_doesnt_exist(layer_idx);
        let layer = &mut self.layers[layer_idx];
        let cursor = layer.rectangles_cursor.get_or_insert_with(|| self.vbo.push_cursor());
        self.vbo.push(*cursor, &SimpleShapeVertex {
            v1: bbox.get_top_left(),
            v2: bbox.get_top_right(),
            v3: bbox.get_bottom_right(),
            color, depth,
        });
    }

    pub fn send_triangle(
        &mut self,
        v1: glm::Vec2,
        v2: glm::Vec2,
        v3: glm::Vec2,
        color: glm::U8Vec4,
        depth: u16,
        layer_idx: usize,
    ) {
        self.create_layer_if_doesnt_exist(layer_idx);
        let layer = &mut self.layers[layer_idx];
        let cursor = layer.triangles_cursor.get_or_insert_with(|| self.vbo.push_cursor());
        self.vbo.push(*cursor, &SimpleShapeVertex {
            v1, v2, v3,
            color, depth,
        });
    }

    pub fn send_ellipse(
        &mut self,
        bbox: &RenderBox,
        color: glm::U8Vec4,
        depth: u16,
        layer_idx: usize,
    ) {
        self.create_layer_if_doesnt_exist(layer_idx);
        let layer = &mut self.layers[layer_idx];
        let cursor = layer.ellipses_cursor.get_or_insert_with(|| self.vbo.push_cursor());
        self.vbo.push(*cursor, &SimpleShapeVertex {
            v1: bbox.get_top_left(),
            v2: bbox.get_top_right(),
            v3: bbox.get_bottom_right(),
            color, depth,
        });
    }
}