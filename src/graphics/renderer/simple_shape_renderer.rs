use std::array;

use macros::{GlVertex, program_interface};
use nalgebra_glm as glm;

use crate::graphics::{camera::Camera, renderer::{Renderer, buffer::{DynamicBuffer, DynamicUnitBuffer, GenericBuffer}, drawable::DrawMode, positioning::{self, OrientedBox, PositionMode, RenderBox}, program::{Program, ShaderType}, uniform::UniformValue, vertex_array_object::{FieldType, StaticVertexLayout, VertexArrayObject}}};

const MAX_FORMS_CAPACITY: usize = 1 << 20;
const BLOCK_SIZE: u16 = 1 << 12;

#[program_interface(
	vert = "../../../res/shaders/rectangle.vert",
	frag = "../../../res/shaders/simple_color.frag"
)]
struct SimpleRectangleProgram {
    u_vp: glm::Mat4,
}

#[program_interface(
	vert = "../../../res/shaders/triangle.vert",
	frag = "../../../res/shaders/simple_color.frag"
)]
struct SimpleTriangleProgram {
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

    fn reset(&mut self) {
        self.block_amount = 0;
        self.current_block_pos = 0;
    }

    fn push<'a>(
        &mut self,
        shape: &ShapeVertex,
        vbo: &mut DynamicUnitBuffer<'a, ShapeVertex>,
        allocated_blocks: &mut u32,
    ) {
        if self.block_amount == 0 || self.current_block_pos == BLOCK_SIZE {
            self.blocks[self.block_amount as usize] = *allocated_blocks as u8;
            self.block_amount += 1;
            *allocated_blocks += 1;
            self.current_block_pos = 0;
        }

        let position = self.blocks[(self.block_amount - 1) as usize] as usize * BLOCK_SIZE as usize + self.current_block_pos as usize;
        self.current_block_pos += 1;
        vbo.write(position, shape);
    }

    fn write_command_buffer<'a>(
        &mut self,
        command_buffer: &mut DynamicBuffer<'a, DrawArraysIndirectCommand>,
        count: u32,
    ) {
        for i in 0..self.block_amount {
            let pos = self.blocks[i as usize] as u32 * BLOCK_SIZE as u32;
            let instance_count = if i + 1 == self.block_amount { self.current_block_pos as u32 } else { BLOCK_SIZE as u32 };
            command_buffer.write(&DrawArraysIndirectCommand {
                count,
                instance_count: instance_count,
                first: pos,
                base_instance: 0,
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

pub struct SimpleShapeRenderer<'a> {
    command_buffer: DynamicBuffer<'a, DrawArraysIndirectCommand>,
    
    rec_program: SimpleRectangleProgram<'a>,
    tri_program: SimpleTriangleProgram<'a>,
    ell_program: SimpleEllipseProgram<'a>,

    vbo: DynamicUnitBuffer<'a, ShapeVertex>,
    vao: VertexArrayObject<'a>,
    allocated_blocks: u32,

    rectangles: RenderGroup,
    transparent_rectangles: RenderGroup,

    triangles: RenderGroup,
    transparent_triangles: RenderGroup,

    ellipses: RenderGroup,
    transparent_ellipses: RenderGroup,
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
            rec_program: SimpleRectangleProgram::init(renderer).unwrap(),
            tri_program: SimpleTriangleProgram::init(renderer).unwrap(),
            ell_program: SimpleEllipseProgram::init(renderer).unwrap(),
            command_buffer: DynamicBuffer::new(renderer, 1 << 8),
            vbo, vao,
            allocated_blocks: 0,
            rectangles: RenderGroup::new(),
            transparent_rectangles: RenderGroup::new(),
            triangles: RenderGroup::new(),
            transparent_triangles: RenderGroup::new(),
            ellipses: RenderGroup::new(),
            transparent_ellipses: RenderGroup::new(),
        }
    }

    pub fn setup_command_buffer(&mut self) {
        self.command_buffer.reset_index();

        self.rectangles.write_command_buffer(&mut self.command_buffer, 4);
        self.triangles.write_command_buffer(&mut self.command_buffer, 3);
        self.ellipses.write_command_buffer(&mut self.command_buffer, 4);

        self.transparent_rectangles.write_command_buffer(&mut self.command_buffer, 4);
        self.transparent_triangles.write_command_buffer(&mut self.command_buffer, 3);
        self.transparent_ellipses.write_command_buffer(&mut self.command_buffer, 4);

        self.command_buffer.flush();
    }

    pub fn draw(&mut self, renderer: &'a Renderer, proj_matrix: &glm::Mat4) {
        self.rec_program.bind();
        self.rec_program.set_u_vp(proj_matrix);
        let (rec_start, rec_len) = (0, self.rectangles.block_amount as u32);
        self.vao.draw_multi_instanced(self.command_buffer.get_id(), rec_start, rec_len, DrawMode::TriangleStrip);

        self.tri_program.bind();
        self.tri_program.set_u_vp(proj_matrix);
        let (tri_start, tri_len) = (rec_start + rec_len, self.triangles.block_amount as u32);
        self.vao.draw_multi_instanced(self.command_buffer.get_id(), tri_start, tri_len, DrawMode::TriangleStrip);
        
        renderer.depth_mask(false);
        self.ell_program.bind();
        self.ell_program.set_u_vp(proj_matrix);
        let (ell_start, ell_len) = (tri_start + tri_len, self.ellipses.block_amount as u32);
        self.vao.draw_multi_instanced(self.command_buffer.get_id(), ell_start, ell_len, DrawMode::TriangleStrip);
        renderer.depth_mask(true);
    }

    pub fn draw_transparent(&mut self, renderer: &'a Renderer, proj_matrix: &glm::Mat4) {
        let (rec_start, rec_len) = (0, self.rectangles.block_amount as u32);
        let (tri_start, tri_len) = (rec_start + rec_len, self.triangles.block_amount as u32);
        let (ell_start, ell_len) = (tri_start + tri_len, self.ellipses.block_amount as u32);

        renderer.depth_mask(false);

        self.rec_program.bind();
        self.rec_program.set_u_vp(proj_matrix);
        let (rec_start, rec_len) = (ell_start + ell_len, self.transparent_rectangles.block_amount as u32);
        self.vao.draw_multi_instanced(self.command_buffer.get_id(), rec_start, rec_len, DrawMode::TriangleStrip);

        self.tri_program.bind();
        self.tri_program.set_u_vp(proj_matrix);
        let (tri_start, tri_len) = (rec_start + rec_len, self.transparent_triangles.block_amount as u32);
        self.vao.draw_multi_instanced(self.command_buffer.get_id(), tri_start, tri_len, DrawMode::TriangleStrip);
        
        self.ell_program.bind();
        self.ell_program.set_u_vp(proj_matrix);
        let (ell_start, ell_len) = (tri_start + tri_len, self.transparent_ellipses.block_amount as u32);
        self.vao.draw_multi_instanced(self.command_buffer.get_id(), ell_start, ell_len, DrawMode::TriangleStrip);
        
        renderer.depth_mask(true);
    }

    pub fn reset(&mut self) {
        self.allocated_blocks = 0;
        println!("{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n", self.rectangles, self.transparent_rectangles, self.triangles, self.transparent_triangles, self.ellipses, self.transparent_ellipses);

        self.rectangles.reset();
        self.transparent_rectangles.reset();
        self.triangles.reset();
        self.transparent_rectangles.reset();
        self.ellipses.reset();
        self.transparent_ellipses.reset();
    }

    pub fn finish(&self) {
        self.vbo.flush_range(0, self.allocated_blocks as usize * BLOCK_SIZE as usize);
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

        let group = if color.w == 255 { &mut self.rectangles } else { &mut self.transparent_rectangles };
        group.push(&ShapeVertex {
            v1: oriented_box.top_left,
            v2: oriented_box.top_right,
            v3: oriented_box.bot_right,
            color, depth,
        }, &mut self.vbo, &mut self.allocated_blocks);
    }

    pub fn preview_render_box(
        position: glm::Vec2,
        size: glm::Vec2,
        position_mode: PositionMode,
    ) -> RenderBox {
        RenderBox::new_simple(position, position_mode, size)
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

        let group = if color.w == 255 { &mut self.rectangles } else { &mut self.transparent_rectangles };
        group.push(&ShapeVertex {
            v1: bbox.get_top_left(),
            v2: bbox.get_top_right(),
            v3: bbox.get_bottom_right(),
            color, depth,
        }, &mut self.vbo, &mut self.allocated_blocks);
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

        let group = if color.w == 255 { &mut self.triangles } else { &mut self.transparent_triangles };
        group.push(&ShapeVertex {
            v1: apex,
            v2: vertex_b,
            v3: vertex_c,
            color, depth,
        }, &mut self.vbo, &mut self.allocated_blocks);
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

        let group = if color.w == 255 { &mut self.triangles } else { &mut self.transparent_triangles };
        group.push(&ShapeVertex {
            v1: vertices[0],
            v2: vertices[1],
            v3: vertices[2],
            color, depth,
        }, &mut self.vbo, &mut self.allocated_blocks);
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

        let group = if color.w == 255 { &mut self.ellipses } else { &mut self.transparent_ellipses };
        group.push(&ShapeVertex {
            v1: oriented_box.top_left,
            v2: oriented_box.top_right,
            v3: oriented_box.bot_right,
            color, depth,
        }, &mut self.vbo, &mut self.allocated_blocks);
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

        let group = if color.w == 255 { &mut self.ellipses } else { &mut self.transparent_ellipses };
        group.push(&ShapeVertex {
            v1: simple_box.get_top_left(),
            v2: simple_box.get_top_right(),
            v3: simple_box.get_bottom_right(),
            color, depth,
        }, &mut self.vbo, &mut self.allocated_blocks);
    }
}
