use std::array;

use macros::{GlVertex, program_interface};
use nalgebra_glm as glm;

use crate::{graphics::{camera::Camera, renderer::{Renderer, buffer::{DynamicBuffer, DynamicUnitBuffer}, positioning::{self, OrientedBox, PositionMode, RenderBox}, program::{Program, ShaderType}, scissor_group::{DrawArraysIndirectCommand, ScissorStack}, uniform::UniformValue, vertex_array_object::{FieldType, StaticVertexLayout, VertexArrayObject}}}, ui::Ui};

pub const MAX_FORMS_CAPACITY: usize = 1 << 20;
pub const BLOCK_SIZE: usize = 1 << 12;

#[program_interface(
	vert = "../../../res/shaders/filled.vert",
	frag = "../../../res/shaders/simple_color.frag"
)]
pub(super) struct SimpleFilledProgram {
    u_vp: glm::Mat4,
}

#[program_interface(
	vert = "../../../res/shaders/ellipse.vert",
	frag = "../../../res/shaders/ellipse.frag"
)]
pub(super) struct SimpleEllipseProgram {
    u_vp: glm::Mat4,
}

#[repr(C)]
#[derive(GlVertex, Debug)]
#[vertex(divisor = 1)]
pub(super) struct ShapeVertex {
    pub v1: glm::Vec2,
    pub v2: glm::Vec2,
    pub v3: glm::Vec2,
    #[normalized]
    pub color: glm::U8Vec4,
    #[normalized]
    pub depth: u16,
}

#[derive(Debug)]
pub(super) struct RenderGroup {
    pub blocks: [u8; 13],
    pub block_amount: u8,
    pub current_block_pos: u16,
}

impl RenderGroup {
    pub(super) fn new() -> Self {
        Self { blocks: [0; _], block_amount: 0, current_block_pos: 0 }
    }

    pub(super) fn push<'a, T>(
        &mut self,
        shape: &T,
        vbo: &mut DynamicUnitBuffer<'a, T>,
        allocated_blocks: &mut u32,
    ) {
        if self.block_amount == 0 || self.current_block_pos as usize == BLOCK_SIZE {
            self.blocks[self.block_amount as usize] = *allocated_blocks as u8; 
            self.block_amount += 1;
            *allocated_blocks += 1;
            self.current_block_pos = 0;
        }

        let position = (self.blocks[(self.block_amount - 1) as usize] as usize * BLOCK_SIZE) + self.current_block_pos as usize;
        self.current_block_pos += 1;
        vbo.write(position, shape);
    }

    pub(super) fn write_command_buffer<'a>(
        &self,
        command_buffer: &mut DynamicBuffer<'a, DrawArraysIndirectCommand>,
        count: u32,
    ) {
        for i in 0..self.block_amount {
            let pos = self.blocks[i as usize] as u32 * BLOCK_SIZE as u32;
            let instance_count = if i + 1 == self.block_amount { self.current_block_pos as u32 } else { BLOCK_SIZE as u32 };
            command_buffer.write(&DrawArraysIndirectCommand {
                count,
                instance_count,
                first: 0,
                base_instance: pos,
            });
        }
    }
}

pub struct SimpleShapeRenderer<'a> {
    filled_program: SimpleFilledProgram<'a>,
    ell_program: SimpleEllipseProgram<'a>,

    vbo: DynamicUnitBuffer<'a, ShapeVertex>,
    vao: VertexArrayObject<'a>,
    allocated_blocks: u32,
}

pub trait HasSimpleShapeRenderer<'a> {
    fn get_simple_shape_renderer(&mut self) -> &mut SimpleShapeRenderer<'a>;
    fn get_groups(&mut self) -> &mut ScissorStack<'a>;
    fn get(&mut self) -> (&mut SimpleShapeRenderer<'a>, &mut ScissorStack<'a>);
}

impl<'a> SimpleShapeRenderer<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        let vbo = DynamicUnitBuffer::new_empty_manual_flush(renderer, MAX_FORMS_CAPACITY);
        let vao = VertexArrayObject::new(renderer, &[&vbo]);

        Self {
            filled_program: SimpleFilledProgram::init(renderer).unwrap(),
            ell_program: SimpleEllipseProgram::init(renderer).unwrap(),
            vbo, vao,
            allocated_blocks: 0,
        }
    }

    pub fn draw(&mut self, renderer: &Renderer, ui: &Ui, proj_matrix: &glm::Mat4, groups: &ScissorStack) {
        let mut command_offset = 0;
        for group in groups.get_groups() {
            group.activate_scissor(ui);
            group.draw(
                renderer,
                proj_matrix,
                &self.filled_program,
                &self.ell_program,
                groups.get_command_buffer_id(),
                &self.vao,
                command_offset,
            );
            command_offset += group.get_command_amount();
        }
    }

    pub fn draw_transparent(&mut self, ui: &Ui, groups: &ScissorStack) {
        let mut command_offset = 0;
        for group in groups.get_groups() {
            group.activate_scissor(ui);
            group.draw_transparent(
                &self.filled_program,
                &self.ell_program,
                groups.get_command_buffer_id(),
                &self.vao,
                command_offset,
            );
            command_offset += group.get_command_amount();
        }
    }

    pub fn reset(&mut self) {
        self.allocated_blocks = 0;
    }

    pub fn finish(&mut self) {
        self.vbo.flush_range(0, self.allocated_blocks as usize * BLOCK_SIZE);
    }

    pub fn send_rotated_rectangle(
        &mut self,
        position: glm::Vec2,
        size: glm::Vec2,
        position_mode: PositionMode,
        up_vector: glm::Vec2,
        color: glm::U8Vec4,
        depth: u16,
        recenter: Option<&Camera>,
        groups: &mut ScissorStack,
    ) {
        let oriented_box = OrientedBox::new_rotated(position, position_mode, size, up_vector);
        let oriented_box = if let Some(camera) = recenter {
            oriented_box.recenter(camera)
        } else {
            oriented_box
        };

        if let Some(group) = groups.get_last() {
            group.send_rectangle(&ShapeVertex {
                v1: oriented_box.top_left,
                v2: oriented_box.top_right,
                v3: oriented_box.bot_right,
                color, depth,
            }, &mut self.vbo, &mut self.allocated_blocks);
        }
    }

    pub fn send_render_box(
        &mut self,
        simple_box: &RenderBox,
        color: glm::U8Vec4,
        depth: u16,
        recenter: Option<&Camera>,
        groups: &mut ScissorStack,
    ) {
        let bbox = if let Some(camera) = recenter {
            simple_box.recenter(camera)
        } else {
            *simple_box
        };

        if let Some(group) = groups.get_last() {
            group.send_rectangle(&ShapeVertex {
                v1: bbox.get_top_left(),
                v2: bbox.get_top_right(),
                v3: bbox.get_bottom_right(),
                color, depth,
            }, &mut self.vbo, &mut self.allocated_blocks);
        }
    }

    pub fn send_rectangle(
        &mut self,
        position: glm::Vec2,
        size: glm::Vec2,
        position_mode: PositionMode,
        color: glm::U8Vec4,
        depth: u16,
        recenter: Option<&Camera>,
        groups: &mut ScissorStack,
    ) -> RenderBox {
        let simple_box = RenderBox::new_simple(position, position_mode, size);
        self.send_render_box(&simple_box, color, depth, recenter, groups);
        return simple_box;
    }

    pub fn send_symmetrical_triangle(
        &mut self,
        apex: glm::Vec2,
        centroid: glm::Vec2,
        width: f32,
        color: glm::U8Vec4,
        depth: u16,
        recenter: Option<&Camera>,
        groups: &mut ScissorStack,
    ) {
        let to_centroid = centroid - apex;
        let dist_r = glm::length(&to_centroid);

        let unit_h = to_centroid / dist_r;
        let unit_w = glm::vec2(-unit_h.y, unit_h.x);

        let base_midpoint = apex + (unit_h * (dist_r * 1.5));
        let half_width = width * 0.5;
        let vertex_b = base_midpoint + (unit_w * half_width);
        let vertex_c = base_midpoint - (unit_w * half_width);

        let (apex, vertex_b, vertex_c) = if let Some(camera) = recenter {
            (
                positioning::recenter(apex, camera),
                positioning::recenter(vertex_b, camera),
                positioning::recenter(vertex_c, camera),
            )
        } else {
            (apex, vertex_b, vertex_c)
        };

        if let Some(group) = groups.get_last() {
            group.send_triangle(&ShapeVertex {
                v1: apex,
                v2: vertex_b,
                v3: vertex_c,
                color, depth,
            }, &mut self.vbo, &mut self.allocated_blocks);
        }
    }

    pub fn send_arbitrary_triangle(
        &mut self,
        vertices: &[glm::Vec2; 3],
        color: glm::U8Vec4,
        depth: u16,
        recenter: Option<&Camera>,
        groups: &mut ScissorStack,
    ) {
        let vertices: [_; 3] = array::from_fn(|i| if let Some(camera) = recenter {
            positioning::recenter(vertices[i], camera)
        } else {
            vertices[i]
        });

        if let Some(group) = groups.get_last() {
            group.send_triangle(&ShapeVertex {
                v1: vertices[0],
                v2: vertices[1],
                v3: vertices[2],
                color, depth,
            }, &mut self.vbo, &mut self.allocated_blocks);
        }
    }

    pub fn send_rotated_ellipse(
        &mut self,
        position: glm::Vec2,
        size: glm::Vec2,
        position_mode: PositionMode,
        up_vector: glm::Vec2,
        color: glm::U8Vec4,
        depth: u16,
        recenter: Option<&Camera>,
        groups: &mut ScissorStack,
    ) {
        let oriented_box = OrientedBox::new_rotated(position, position_mode, size, up_vector);
        let oriented_box = if let Some(camera) = recenter {
            oriented_box.recenter(camera)
        } else {
            oriented_box
        };

        if let Some(group) = groups.get_last() {
            group.send_ellipse(&ShapeVertex {
                v1: oriented_box.top_left,
                v2: oriented_box.top_right,
                v3: oriented_box.bot_right,
                color, depth,
            }, &mut self.vbo, &mut self.allocated_blocks);
        }
    }

    pub fn send_ellipse(
        &mut self,
        position: glm::Vec2,
        size: glm::Vec2,
        position_mode: PositionMode,
        color: glm::U8Vec4,
        depth: u16,
        recenter: Option<&Camera>,
        groups: &mut ScissorStack,
    ) {
        let simple_box = RenderBox::new_simple(position, position_mode, size);
        let simple_box = if let Some(camera) = recenter {
            simple_box.recenter(camera)
        } else {
            simple_box
        };

        if let Some(group) = groups.get_last() {
            group.send_ellipse(&ShapeVertex {
                v1: simple_box.get_top_left(),
                v2: simple_box.get_top_right(),
                v3: simple_box.get_bottom_right(),
                color, depth,
            }, &mut self.vbo, &mut self.allocated_blocks);
        }
    }
}
