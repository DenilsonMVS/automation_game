use std::num::NonZeroU32;

use macros::{GlVertex, program_interface};
use nalgebra_glm as glm;
use anyhow::Result;
use crate::graphics::renderer::{buffer::{block_buffer::BlockBuffer, stream_buffer::StreamBuffer}, draw_mode::DrawMode, fonts::font::{Font, PX_RANGE}, positioning::PositionMode, program::{Program, ShaderType, uniform::UniformValue}, vertex_array_object::{DrawArraysIndirectCommand, FieldType, StaticVertexLayout, VertexArrayObject}};

pub(crate) mod font;

#[derive(Debug, Clone, Copy)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Copy)]
pub struct TextRenderConfig<'a> {
    pub text: &'a [(&'a str, glm::U8Vec4)],
    pub position: glm::Vec2,
    pub line_height: f32,
    pub max_line_width: f32,
    pub forward: glm::Vec2,
    pub position_mode: PositionMode,
    pub depth: u16,
    pub align: TextAlign,
}

impl<'a> TextRenderConfig<'a> {
    pub fn new(text: &'a [(&'a str, glm::U8Vec4)], position: glm::Vec2, line_height: f32) -> Self {
        Self {
            text,
            position,
            line_height,
            position_mode: PositionMode::Center,
            forward: glm::vec2(1.0, 0.0),
            depth: 1000,
            max_line_width: 1e2,
            align: TextAlign::Left,
        }
    }

    pub fn with_mode(mut self, mode: PositionMode) -> Self {
        self.position_mode = mode;
        return self;
    }

    pub fn with_depth(mut self, depth: u16) -> Self {
        self.depth = depth;
        return self;
    }

    pub fn with_forward(mut self, forward: glm::Vec2) -> Self {
        self.forward = forward;
        return self;
    }

    pub fn with_max_line_width(mut self, max_line_width: f32) -> Self {
        self.max_line_width = max_line_width;
        return self;
    }

    pub fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        return self;
    }
}

#[repr(C)]
#[derive(GlVertex, Debug)]
#[vertex(divisor = 1)]
pub(crate) struct GlyphAttrs {
    top_left: glm::Vec2,
    top_right: glm::Vec2,
    #[normalized]
    color: glm::U8Vec4,
    #[normalized]
    depth: u16,
    character_idx: u8,
}

#[program_interface(
    vert = "../../../../res/shaders/font.vert",
    frag = "../../../../res/shaders/font.frag"
)]
struct FontProgram {
    u_texture: i32,
    u_projection: glm::Mat4,
    u_px_range: f32,
}

pub struct FontRenderer {
    font_program: FontProgram,
    glyph_buffer: BlockBuffer<GlyphAttrs>,
    glyph_vao: VertexArrayObject,
    command_buffer: StreamBuffer<DrawArraysIndirectCommand>,
}

impl FontRenderer {
    pub(crate) fn new() -> Result<Self> {
        let font_program = FontProgram::init()?;
        font_program.set_u_texture(&0);
        font_program.set_u_px_range(&PX_RANGE);

        let glyph_buffer = BlockBuffer::<GlyphAttrs>::new();
        let glyph_vao = VertexArrayObject::new(&[&glyph_buffer]);
        let command_buffer = StreamBuffer::new(NonZeroU32::new(0x100).unwrap());

        Ok(Self {
            font_program,
            glyph_buffer,
            glyph_vao,
            command_buffer,
        })
    }

    pub fn reset(&mut self) {
        self.glyph_buffer.reset();
        self.command_buffer.reset();
    }

    pub fn push(&mut self, text: &TextRenderConfig, font: &mut Font, layer_idx: usize) {
        font.add_text(text, &mut self.glyph_buffer, layer_idx);
    }

    pub(crate) fn draw(&mut self, proj_matrix: &glm::Mat4, font: &mut Font, layer_idx: usize) {
        let Some(cursor) = font.get_cursor(layer_idx) else { return };

        let command_start = self.command_buffer.get_cursor_pos();
        for block in self.glyph_buffer.get_cursor_blocks(cursor) {
            self.command_buffer.push(&DrawArraysIndirectCommand {
                count: 4,
                instance_count: block.used_size,
                first: 0,
                base_instance: block.block_start,
            });
        }

        let command_count = self.command_buffer.get_cursor_pos() - command_start;
        if command_count == 0 { return };

        self.font_program.bind();
        self.font_program.set_u_projection(proj_matrix);

        if self.glyph_buffer.take_rebind() {
            self.glyph_vao.rebind(0, &self.glyph_buffer);
        }

        font.bind_to_unit(0);
        font.bind_uv_storage();

        unsafe { gl::DepthMask(gl::FALSE); }
        self.glyph_vao.draw_multi_instanced(
            &self.command_buffer,
            command_start,
            command_count,
            DrawMode::TriangleStrip,
        );
        unsafe { gl::DepthMask(gl::TRUE); }
    }
}
