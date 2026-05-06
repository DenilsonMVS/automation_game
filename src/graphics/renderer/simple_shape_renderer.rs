use std::array;

use macros::{GlVertex, program_interface};
use nalgebra_glm as glm;

use crate::{graphics::{camera::Camera, renderer::{Renderer, buffer::{DynamicBuffer, DynamicUnitBuffer, GenericBuffer}, drawable::DrawMode, positioning::{self, OrientedBox, PositionMode, RenderBox}, program::{Program, ShaderType}, uniform::UniformValue, vertex_array_object::{FieldType, StaticVertexLayout, VertexArrayObject}}}, ui::Ui};

const MAX_FORMS_CAPACITY: usize = 1 << 20;
const BLOCK_SIZE: usize = 1 << 12;

#[program_interface(
	vert = "../../../res/shaders/filled.vert",
	frag = "../../../res/shaders/simple_color.frag"
)]
struct SimpleFilledProgram {
    u_vp: glm::Mat4,
}

#[program_interface(
	vert = "../../../res/shaders/ellipse.vert",
	frag = "../../../res/shaders/ellipse.frag"
)]
struct SimpleEllipseProgram {
    u_vp: glm::Mat4,
}

#[repr(C)]
#[derive(GlVertex, Debug)]
#[vertex(divisor = 1)]
struct ShapeVertex {
    v1: glm::Vec2,
    v2: glm::Vec2,
    v3: glm::Vec2,
    #[normalized]
    color: glm::U8Vec4,
    #[normalized]
    depth: u16,
}

#[derive(Debug)]
struct RenderGroup {
    blocks: [u8; 13],
    block_amount: u8,
    current_block_pos: u16,
}

impl RenderGroup {
    fn new() -> Self {
        Self { blocks: [0; _], block_amount: 0, current_block_pos: 0 }
    }

    fn push<'a>(
        &mut self,
        shape: &ShapeVertex,
        vbo: &mut DynamicUnitBuffer<'a, ShapeVertex>,
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

    fn write_command_buffer<'a>(
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

#[repr(C)]
pub struct DrawArraysIndirectCommand {
    pub count: u32,
    pub instance_count: u32,
    pub first: u32,
    pub base_instance: u32,
}

pub struct ScissorCutGroup {
    bbox: RenderBox,

    rectangles: RenderGroup,
    transparent_rectangles: RenderGroup,

    triangles: RenderGroup,
    transparent_triangles: RenderGroup,

    ellipses: RenderGroup,
    transparent_ellipses: RenderGroup,
}

impl ScissorCutGroup {
    fn new(bbox: RenderBox) -> Self {
        Self {
            bbox,
            rectangles: RenderGroup::new(),
            transparent_rectangles: RenderGroup::new(),
            triangles: RenderGroup::new(),
            transparent_triangles: RenderGroup::new(),
            ellipses: RenderGroup::new(),
            transparent_ellipses: RenderGroup::new(),
        }
    }

    fn draw<'a>(
        &self,
        renderer: &Renderer,
        proj_matrix: &glm::Mat4,
        filled_program: &SimpleFilledProgram<'a>,
        ell_program: &SimpleEllipseProgram<'a>,
        command_buffer_id: u32,
        vao: &VertexArrayObject,
    ) {
        filled_program.bind();
        filled_program.set_u_vp(proj_matrix);
        let (rec_start, rec_len) = (0, self.rectangles.block_amount as u32);
        let (tri_start, tri_len) = (rec_start + rec_len, self.triangles.block_amount as u32);
        vao.draw_multi_instanced(command_buffer_id, rec_start, rec_len + tri_len, DrawMode::TriangleStrip);

        renderer.depth_mask(false);
        ell_program.bind();
        ell_program.set_u_vp(proj_matrix);
        let (ell_start, ell_len) = (tri_start + tri_len, self.ellipses.block_amount as u32);
        vao.draw_multi_instanced(command_buffer_id, ell_start, ell_len, DrawMode::TriangleStrip);
        renderer.depth_mask(true);
    }

    fn draw_transparent<'a>(
        &self,
        renderer: &Renderer,
        filled_program: &SimpleFilledProgram<'a>,
        ell_program: &SimpleEllipseProgram<'a>,
        command_buffer_id: u32,
        vao: &VertexArrayObject,
    ) {
        let (rec_start, rec_len) = (0, self.rectangles.block_amount as u32);
        let (tri_start, tri_len) = (rec_start + rec_len, self.triangles.block_amount as u32);
        let (ell_start, ell_len) = (tri_start + tri_len, self.ellipses.block_amount as u32);

        renderer.depth_mask(false);

        filled_program.bind();
        let (rec_start, rec_len) = (ell_start + ell_len, self.transparent_rectangles.block_amount as u32);
        let (tri_start, tri_len) = (rec_start + rec_len, self.transparent_triangles.block_amount as u32);
        vao.draw_multi_instanced(command_buffer_id, rec_start, rec_len + tri_len, DrawMode::TriangleStrip);
        
        ell_program.bind();
        let (ell_start, ell_len) = (tri_start + tri_len, self.transparent_ellipses.block_amount as u32);
        vao.draw_multi_instanced(command_buffer_id, ell_start, ell_len, DrawMode::TriangleStrip);
        
        renderer.depth_mask(true);
    }

    fn finish<'a>(&self, command_buffer: &mut DynamicBuffer<'a, DrawArraysIndirectCommand>) {
        self.rectangles.write_command_buffer(command_buffer, 4);
        self.triangles.write_command_buffer(command_buffer, 3);
        self.ellipses.write_command_buffer(command_buffer, 4);

        self.transparent_rectangles.write_command_buffer(command_buffer, 4);
        self.transparent_triangles.write_command_buffer(command_buffer, 3);
        self.transparent_ellipses.write_command_buffer(command_buffer, 4);
    }

    fn send_rectangle<'a>(
        &mut self,
        vertex: &ShapeVertex,
        vbo: &mut DynamicUnitBuffer<'a, ShapeVertex>,
        allocated_blocks: &mut u32,
    ) {
        let group = if vertex.color.w == 255 { &mut self.rectangles } else { &mut self.transparent_rectangles };
        group.push(vertex, vbo, allocated_blocks);
    }

    fn send_triangle<'a>(
        &mut self,
        vertex: &ShapeVertex,
        vbo: &mut DynamicUnitBuffer<'a, ShapeVertex>,
        allocated_blocks: &mut u32,
    ) {
        let group = if vertex.color.w == 255 { &mut self.triangles } else { &mut self.transparent_triangles };
        group.push(vertex, vbo, allocated_blocks);
    }

    fn send_ellipse<'a>(
        &mut self,
        vertex: &ShapeVertex,
        vbo: &mut DynamicUnitBuffer<'a, ShapeVertex>,
        allocated_blocks: &mut u32,
    ) {
        let group = if vertex.color.w == 255 { &mut self.ellipses } else { &mut self.transparent_ellipses };
        group.push(vertex, vbo, allocated_blocks);
    }

    fn activate_scissor(&self, ui: &Ui) {
        let bot_left = ui.ndc_to_screen_position(self.bbox.get_bottom_left()).map(|v| v.round() as i32);
        let top_right = ui.ndc_to_screen_position(self.bbox.get_top_right()).map(|v| v.round() as i32);
        let size = top_right - bot_left;
        unsafe { gl::Scissor(bot_left.x, bot_left.y, size.x, size.y); }
    }
}

pub struct SimpleShapeRenderer<'a> {
    command_buffer: DynamicBuffer<'a, DrawArraysIndirectCommand>,
    
    filled_program: SimpleFilledProgram<'a>,
    ell_program: SimpleEllipseProgram<'a>,

    vbo: DynamicUnitBuffer<'a, ShapeVertex>,
    vao: VertexArrayObject<'a>,
    allocated_blocks: u32,

    scissor_groups: Vec<ScissorCutGroup>,
    scissor_group_index_stack: Vec<u8>,
}

pub trait HasSimpleShapeRenderer<'a> {
    fn get_simple_shape_renderer(&mut self) -> &mut SimpleShapeRenderer<'a>;
}

impl<'a> HasSimpleShapeRenderer<'a> for SimpleShapeRenderer<'a> {
    fn get_simple_shape_renderer(&mut self) -> &mut SimpleShapeRenderer<'a> {
        self
    }
}

impl<'a> SimpleShapeRenderer<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        let vbo = DynamicUnitBuffer::new_empty_manual_flush(renderer, MAX_FORMS_CAPACITY);
        let vao = VertexArrayObject::new(renderer, &[&vbo]);

        Self {
            filled_program: SimpleFilledProgram::init(renderer).unwrap(),
            ell_program: SimpleEllipseProgram::init(renderer).unwrap(),
            command_buffer: DynamicBuffer::new(renderer, 1 << 8),
            vbo, vao,
            allocated_blocks: 0,
            scissor_groups: vec![],
            scissor_group_index_stack: vec![],
        }
    }

    pub fn push_scissor_group(&mut self, bbox: RenderBox) {
        self.scissor_group_index_stack.push(self.scissor_groups.len() as u8);
        self.scissor_groups.push(ScissorCutGroup::new(bbox));
    }

    pub fn pop_scissor_group(&mut self) {
        self.scissor_group_index_stack.pop();
    }

    pub fn draw(&mut self, renderer: &Renderer, ui: &Ui, proj_matrix: &glm::Mat4) {
        for group in self.scissor_groups.iter() {
            group.activate_scissor(ui);
            group.draw(
                renderer,
                proj_matrix,
                &self.filled_program,
                &self.ell_program,
                self.command_buffer.get_id(),
                &self.vao,
            );
        }
    }

    pub fn draw_transparent(&mut self, renderer: &'a Renderer, ui: &Ui) {
        for group in self.scissor_groups.iter() {
            group.activate_scissor(ui);
            group.draw_transparent(
                renderer,
                &self.filled_program,
                &self.ell_program,
                self.command_buffer.get_id(),
                &self.vao,
            );
        }
    }

    pub fn reset(&mut self) {
        self.allocated_blocks = 0;
        self.scissor_group_index_stack.clear();
        self.scissor_groups.clear();
    }

    pub fn finish(&mut self) {
        self.command_buffer.reset_index();

        for group in self.scissor_groups.iter() {
            group.finish(&mut self.command_buffer);
        }

        self.vbo.flush_range(0, self.allocated_blocks as usize * BLOCK_SIZE);
        self.command_buffer.flush();
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
    ) {
        let oriented_box = OrientedBox::new_rotated(position, position_mode, size, up_vector);
        let oriented_box = if let Some(camera) = recenter {
            oriented_box.recenter(camera)
        } else {
            oriented_box
        };

        if
            let Some(idx) = self.scissor_group_index_stack.last() &&
            let Some(group) = self.scissor_groups.get_mut(*idx as usize)
        {
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
    ) {
        let bbox = if let Some(camera) = recenter {
            simple_box.recenter(camera)
        } else {
            *simple_box
        };

        if
            let Some(idx) = self.scissor_group_index_stack.last() &&
            let Some(group) = self.scissor_groups.get_mut(*idx as usize)
        {
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
    ) -> RenderBox {
        let simple_box = RenderBox::new_simple(position, position_mode, size);
        self.send_render_box(&simple_box, color, depth, recenter);
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

        if
            let Some(idx) = self.scissor_group_index_stack.last() &&
            let Some(group) = self.scissor_groups.get_mut(*idx as usize)
        {
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
    ) {
        let vertices: [_; 3] = array::from_fn(|i| if let Some(camera) = recenter {
            positioning::recenter(vertices[i], camera)
        } else {
            vertices[i]
        });

        if
            let Some(idx) = self.scissor_group_index_stack.last() &&
            let Some(group) = self.scissor_groups.get_mut(*idx as usize)
        {
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
    ) {
        let oriented_box = OrientedBox::new_rotated(position, position_mode, size, up_vector);
        let oriented_box = if let Some(camera) = recenter {
            oriented_box.recenter(camera)
        } else {
            oriented_box
        };

        if
            let Some(idx) = self.scissor_group_index_stack.last() &&
            let Some(group) = self.scissor_groups.get_mut(*idx as usize)
        {
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
    ) {
        let simple_box = RenderBox::new_simple(position, position_mode, size);
        let simple_box = if let Some(camera) = recenter {
            simple_box.recenter(camera)
        } else {
            simple_box
        };

        if
            let Some(idx) = self.scissor_group_index_stack.last() &&
            let Some(group) = self.scissor_groups.get_mut(*idx as usize)
        {
            group.send_ellipse(&ShapeVertex {
                v1: simple_box.get_top_left(),
            v2: simple_box.get_top_right(),
            v3: simple_box.get_bottom_right(),
                color, depth,
            }, &mut self.vbo, &mut self.allocated_blocks);
        }
    }
}
