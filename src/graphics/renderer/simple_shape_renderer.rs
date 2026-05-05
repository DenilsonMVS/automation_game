use std::array;

use macros::{GlVertex, program_interface};
use nalgebra_glm as glm;

use crate::graphics::{camera::Camera, renderer::{Renderer, buffer::{DynamicBuffer, GenericBuffer}, drawable::DrawMode, positioning::{self, OrientedBox, PositionMode, RenderBox}, program::{Program, ShaderType}, uniform::UniformValue, vertex_array_object::{FieldType, StaticVertexLayout, VertexArrayObject}}};

const FORMS_PER_RENDER: usize = 1 << 20;

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
struct RenderGroup<'a> {
    pub vbo: DynamicBuffer<'a, ShapeVertex>,
    pub vao: VertexArrayObject<'a>,
}

impl<'a> RenderGroup<'a> {
    fn new(renderer: &'a Renderer) -> Self {
        let vbo = DynamicBuffer::new(renderer, FORMS_PER_RENDER);
        let vao = VertexArrayObject::new(renderer, &[&vbo]);
        Self { vbo, vao }
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
    rec_buff: RenderGroup<'a>,
    rec_buff_trans: RenderGroup<'a>,

    tri_program: SimpleTriangleProgram<'a>,
    tri_buff: RenderGroup<'a>,
    tri_buff_trans: RenderGroup<'a>,

    ell_program: SimpleEllipseProgram<'a>,
    ell_buff: RenderGroup<'a>,
    ell_buff_trans: RenderGroup<'a>,
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
        Self {
            command_buffer: DynamicBuffer::new(renderer, 1 << 8),

            rec_program: SimpleRectangleProgram::init(renderer).unwrap(),
            rec_buff: RenderGroup::new(renderer),
            rec_buff_trans: RenderGroup::new(renderer),

            tri_program: SimpleTriangleProgram::init(renderer).unwrap(),
            tri_buff: RenderGroup::new(renderer),
            tri_buff_trans: RenderGroup::new(renderer),

            ell_program: SimpleEllipseProgram::init(renderer).unwrap(),
            ell_buff: RenderGroup::new(renderer),
            ell_buff_trans: RenderGroup::new(renderer),
        }
    }

    pub fn setup_command_buffer(&mut self) {
        self.command_buffer.reset_index();

        let RenderGroup { vbo, ..} = &mut self.rec_buff;
        self.command_buffer.write(&DrawArraysIndirectCommand {
            count: 4, instance_count: vbo.get_len() as u32, first: 0, base_instance: 0 });

        let RenderGroup { vbo, ..} = &mut self.tri_buff;
        self.command_buffer.write(&DrawArraysIndirectCommand {
            count: 3, instance_count: vbo.get_len() as u32, first: 0, base_instance: 0 });

        let RenderGroup { vbo, ..} = &mut self.ell_buff;
        self.command_buffer.write(&DrawArraysIndirectCommand {
            count: 4, instance_count: vbo.get_len() as u32, first: 0, base_instance: 0 });

        let RenderGroup { vbo, ..} = &mut self.rec_buff_trans;
        self.command_buffer.write(&DrawArraysIndirectCommand {
            count: 4, instance_count: vbo.get_len() as u32, first: 0, base_instance: 0 });

        let RenderGroup { vbo, ..} = &mut self.tri_buff_trans;
        self.command_buffer.write(&DrawArraysIndirectCommand {
            count: 3, instance_count: vbo.get_len() as u32, first: 0, base_instance: 0 });

        let RenderGroup { vbo, ..} = &mut self.ell_buff_trans;
        self.command_buffer.write(&DrawArraysIndirectCommand {
            count: 4, instance_count: vbo.get_len() as u32, first: 0, base_instance: 0 });

        self.command_buffer.flush();
    }

    pub fn draw(&mut self, renderer: &'a Renderer, proj_matrix: &glm::Mat4) {
        self.rec_program.bind();
        self.rec_program.set_u_vp(proj_matrix);
        let RenderGroup { vao, ..} = &mut self.rec_buff;
        vao.draw_multi_instanced(self.command_buffer.get_id(), 0, 1, DrawMode::TriangleStrip);

        self.tri_program.bind();
        self.tri_program.set_u_vp(proj_matrix);
        let RenderGroup { vao, ..} = &mut self.tri_buff;
        vao.draw_multi_instanced(self.command_buffer.get_id(), 1, 1, DrawMode::Triangles);
        
        renderer.depth_mask(false);
        self.ell_program.bind();
        self.ell_program.set_u_vp(proj_matrix);
        let RenderGroup { vao, ..} = &mut self.ell_buff;
        vao.draw_multi_instanced(self.command_buffer.get_id(), 2, 1, DrawMode::TriangleStrip);
        renderer.depth_mask(true);
    }

    pub fn draw_transparent(&mut self, renderer: &'a Renderer, proj_matrix: &glm::Mat4) {
        renderer.depth_mask(false);

        self.rec_program.bind();
        self.rec_program.set_u_vp(proj_matrix);
        let RenderGroup { vao, ..} = &mut self.rec_buff_trans;
        vao.draw_multi_instanced(self.command_buffer.get_id(), 3, 1, DrawMode::TriangleStrip);

        self.tri_program.bind();
        self.tri_program.set_u_vp(proj_matrix);
        let RenderGroup { vao, ..} = &mut self.tri_buff_trans;
        vao.draw_multi_instanced(self.command_buffer.get_id(), 4, 1, DrawMode::Triangles);
        
        self.ell_program.bind();
        self.ell_program.set_u_vp(proj_matrix);
        let RenderGroup { vao, ..} = &mut self.ell_buff_trans;
        vao.draw_multi_instanced(self.command_buffer.get_id(), 5, 1, DrawMode::TriangleStrip);
        
        renderer.depth_mask(true);
    }

    fn get_rec_buffer(&mut self, alpha: u8) -> &mut DynamicBuffer<'a, ShapeVertex> {
        if alpha == 255 { &mut self.rec_buff.vbo } else { &mut self.rec_buff_trans.vbo }
    }

    fn get_tri_buffer(&mut self, alpha: u8) -> &mut DynamicBuffer<'a, ShapeVertex> {
        if alpha == 255 { &mut self.tri_buff.vbo } else { &mut self.tri_buff_trans.vbo }
    }

    fn get_ell_buffer(&mut self, alpha: u8) -> &mut DynamicBuffer<'a, ShapeVertex> {
        if alpha == 255 { &mut self.ell_buff.vbo } else { &mut self.ell_buff_trans.vbo }
    }

    pub fn reset(&mut self) {
        self.rec_buff.vbo.reset_index();
        self.rec_buff_trans.vbo.reset_index();
        self.tri_buff.vbo.reset_index();
        self.tri_buff_trans.vbo.reset_index();
        self.ell_buff.vbo.reset_index();
        self.ell_buff_trans.vbo.reset_index();
    }

    pub fn finish(&self) {
        self.rec_buff.vbo.flush();
        self.rec_buff_trans.vbo.flush();
        self.tri_buff.vbo.flush();
        self.tri_buff_trans.vbo.flush();
        self.ell_buff.vbo.flush();
        self.ell_buff_trans.vbo.flush();
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
        let buffer = self.get_rec_buffer(color.w);
        buffer.write(&ShapeVertex {
            v1: oriented_box.top_left,
            v2: oriented_box.top_right,
            v3: oriented_box.bot_right,
            color, depth,
        });
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
        let buffer = self.get_rec_buffer(color.w);
        buffer.write(&ShapeVertex {
            v1: bbox.get_top_left(),
            v2: bbox.get_top_right(),
            v3: bbox.get_bottom_right(),
            color, depth,
        });
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

        let buffer = self.get_tri_buffer(color.w);
        buffer.write(&ShapeVertex {
            v1: apex,
            v2: vertex_b,
            v3: vertex_c,
            color,
            depth,
        });
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

        let buffer = self.get_tri_buffer(color.w);
        buffer.write(&ShapeVertex {
            v1: vertices[0],
            v2: vertices[1],
            v3: vertices[2],
            color,
            depth,
        });
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

        let buffer = self.get_ell_buffer(color.w);
        buffer.write(&ShapeVertex {
            v1: oriented_box.top_left,
            v2: oriented_box.top_right,
            v3: oriented_box.bot_right,
            color, depth,
        });
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

        let buffer = self.get_ell_buffer(color.w);
        buffer.write(&ShapeVertex {
            v1: simple_box.get_top_left(),
            v2: simple_box.get_top_right(),
            v3: simple_box.get_bottom_right(),
            color, depth,
        });
    }
}
