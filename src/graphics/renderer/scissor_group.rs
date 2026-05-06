use crate::{graphics::renderer::{Renderer, buffer::{DynamicBuffer, DynamicUnitBuffer, GenericBuffer}, drawable::DrawMode, fonts::GlyphAttrs, positioning::RenderBox, simple_shape_renderer::{RenderGroup, ShapeVertex, SimpleEllipseProgram, SimpleFilledProgram}, vertex_array_object::VertexArrayObject}, ui::Ui};
use nalgebra_glm as glm;

#[repr(C)]
pub struct DrawArraysIndirectCommand {
    pub count: u32,
    pub instance_count: u32,
    pub first: u32,
    pub base_instance: u32,
}

pub(super) struct ScissorCutGroup {
    bbox: RenderBox,

    fonts: RenderGroup,

    rectangles: RenderGroup,
    transparent_rectangles: RenderGroup,

    triangles: RenderGroup,
    transparent_triangles: RenderGroup,

    ellipses: RenderGroup,
    transparent_ellipses: RenderGroup,
}

impl ScissorCutGroup {
    pub(super) fn new(bbox: RenderBox) -> Self {
        Self {
            bbox,
            fonts: RenderGroup::new(),
            rectangles: RenderGroup::new(),
            transparent_rectangles: RenderGroup::new(),
            triangles: RenderGroup::new(),
            transparent_triangles: RenderGroup::new(),
            ellipses: RenderGroup::new(),
            transparent_ellipses: RenderGroup::new(),
        }
    }

    pub(super) fn draw<'a>(
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

    pub(super) fn draw_transparent<'a>(
        &self,
        filled_program: &SimpleFilledProgram<'a>,
        ell_program: &SimpleEllipseProgram<'a>,
        command_buffer_id: u32,
        vao: &VertexArrayObject,
    ) {
        let (rec_start, rec_len) = (0, self.rectangles.block_amount as u32);
        let (tri_start, tri_len) = (rec_start + rec_len, self.triangles.block_amount as u32);
        let (ell_start, ell_len) = (tri_start + tri_len, self.ellipses.block_amount as u32);

        filled_program.bind();
        let (rec_start, rec_len) = (ell_start + ell_len, self.transparent_rectangles.block_amount as u32);
        let (tri_start, tri_len) = (rec_start + rec_len, self.transparent_triangles.block_amount as u32);
        vao.draw_multi_instanced(command_buffer_id, rec_start, rec_len + tri_len, DrawMode::TriangleStrip);
        
        ell_program.bind();
        let (ell_start, ell_len) = (tri_start + tri_len, self.transparent_ellipses.block_amount as u32);
        vao.draw_multi_instanced(command_buffer_id, ell_start, ell_len, DrawMode::TriangleStrip);
    }

    pub(super) fn draw_font(
        &self,
        vao: &VertexArrayObject,
        command_buffer_id: u32,
    ) {
        let (rec_start, rec_len) = (0, self.rectangles.block_amount as u32);
        let (tri_start, tri_len) = (rec_start + rec_len, self.triangles.block_amount as u32);
        let (ell_start, ell_len) = (tri_start + tri_len, self.ellipses.block_amount as u32);
        let (rec_start, rec_len) = (ell_start + ell_len, self.transparent_rectangles.block_amount as u32);
        let (tri_start, tri_len) = (rec_start + rec_len, self.transparent_triangles.block_amount as u32);
        let (ell_start, ell_len) = (tri_start + tri_len, self.transparent_ellipses.block_amount as u32);
        let (font_start, font_len) = (ell_start + ell_len, self.fonts.block_amount as u32);

        vao.draw_multi_instanced(command_buffer_id, font_start, font_len, DrawMode::TriangleStrip);
    }

    pub(super) fn finish<'a>(&self, command_buffer: &mut DynamicBuffer<'a, DrawArraysIndirectCommand>) {
        self.rectangles.write_command_buffer(command_buffer, 4);
        self.triangles.write_command_buffer(command_buffer, 3);
        self.ellipses.write_command_buffer(command_buffer, 4);

        self.transparent_rectangles.write_command_buffer(command_buffer, 4);
        self.transparent_triangles.write_command_buffer(command_buffer, 3);
        self.transparent_ellipses.write_command_buffer(command_buffer, 4);

        self.fonts.write_command_buffer(command_buffer, 4);
    }

    pub(super)fn send_rectangle<'a>(
        &mut self,
        vertex: &ShapeVertex,
        vbo: &mut DynamicUnitBuffer<'a, ShapeVertex>,
        allocated_blocks: &mut u32,
    ) {
        let group = if vertex.color.w == 255 { &mut self.rectangles } else { &mut self.transparent_rectangles };
        group.push(vertex, vbo, allocated_blocks);
    }

    pub(super) fn send_triangle<'a>(
        &mut self,
        vertex: &ShapeVertex,
        vbo: &mut DynamicUnitBuffer<'a, ShapeVertex>,
        allocated_blocks: &mut u32,
    ) {
        let group = if vertex.color.w == 255 { &mut self.triangles } else { &mut self.transparent_triangles };
        group.push(vertex, vbo, allocated_blocks);
    }

    pub(super) fn send_ellipse<'a>(
        &mut self,
        vertex: &ShapeVertex,
        vbo: &mut DynamicUnitBuffer<'a, ShapeVertex>,
        allocated_blocks: &mut u32,
    ) {
        let group = if vertex.color.w == 255 { &mut self.ellipses } else { &mut self.transparent_ellipses };
        group.push(vertex, vbo, allocated_blocks);
    }

    pub(super) fn send_text<'a>(
        &mut self,
        vertex: &GlyphAttrs,
        vbo: &mut DynamicUnitBuffer<'a, GlyphAttrs>,
        allocated_blocks: &mut u32
    ) {
        self.fonts.push(vertex, vbo, allocated_blocks);
    }

    pub(super) fn activate_scissor(&self, ui: &Ui) {
        let bot_left = ui.ndc_to_screen_position(self.bbox.get_bottom_left()).map(|v| v.round() as i32);
        let top_right = ui.ndc_to_screen_position(self.bbox.get_top_right()).map(|v| v.round() as i32);
        let size = top_right - bot_left;
        unsafe { gl::Scissor(bot_left.x, bot_left.y, size.x, size.y); }
    }
}

pub struct ScissorStack<'a> {
    command_buffer: DynamicBuffer<'a, DrawArraysIndirectCommand>,
    groups: Vec<ScissorCutGroup>,
    index_stack: Vec<usize>,
}

impl<'a> ScissorStack<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        Self {
            command_buffer: DynamicBuffer::new(renderer, 1 << 8),
            groups: vec![],
            index_stack: vec![],
        }
    }

    pub fn push(&mut self, bbox: RenderBox) {
        self.index_stack.push(self.groups.len());
        self.groups.push(ScissorCutGroup::new(bbox));
    }

    pub fn pop(&mut self) {
        self.groups.pop();
    }

    pub fn reset(&mut self) {
        self.groups.clear();
        self.index_stack.clear();
    }

    pub fn finish(&mut self) {
        self.command_buffer.reset_index();

        for group in self.groups.iter() {
            group.finish(&mut self.command_buffer);
        }

        self.command_buffer.flush();
    }

    pub fn get_command_buffer_id(&self) -> u32 {
        self.command_buffer.get_id()
    }

    pub(super) fn get_last(&mut self) -> Option<&mut ScissorCutGroup> {
        let idx = self.index_stack.last()?;
        self.groups.get_mut(*idx)
    }

    pub(super) fn get_groups(&self) -> &[ScissorCutGroup] {
        &self.groups
    }
}