use std::array;

use macros::{GlVertex, program_interface};
use nalgebra_glm as glm;

use crate::graphics::{camera::Camera, renderer::{Renderer, buffer::DynamicBuffer, drawable::DrawMode, positioning::{self, OrientedBox, PositionMode, RenderBox}, program::{Program, ShaderType}, uniform::UniformValue, vertex_array_object::{FieldType, StaticVertexLayout, VertexArrayObject}}};

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
struct RectangleVertex {
    top_left: glm::Vec2,
    top_right: glm::Vec2,
    bottom_right: glm::Vec2,
    #[normalized]
    color: glm::U8Vec4,
    #[normalized]
    depth: u16,
}

#[repr(C)]
#[derive(GlVertex, Debug)]
#[vertex(divisor = 1)]
struct TriangleVertex {
    v1: glm::Vec2,
    v2: glm::Vec2,
    v3: glm::Vec2,
    #[normalized]
    color: glm::U8Vec4,
    #[normalized]
    depth: u16,
}

#[repr(C)]
#[derive(GlVertex, Debug)]
#[vertex(divisor = 1)]
struct EllipseVertex {
    top_left: glm::Vec2,
    top_right: glm::Vec2,
    bottom_right: glm::Vec2,
    #[normalized]
    color: glm::U8Vec4,
    #[normalized]
    depth: u16,
}

pub struct SimpleShapeRenderer<'a> {
    rec_program: SimpleRectangleProgram<'a>,
    rec_buff: (DynamicBuffer<'a, RectangleVertex>, VertexArrayObject<'a>),
    rec_buff_trans: (DynamicBuffer<'a, RectangleVertex>, VertexArrayObject<'a>),

    tri_program: SimpleTriangleProgram<'a>,
    tri_buff: (DynamicBuffer<'a, TriangleVertex>, VertexArrayObject<'a>),
    tri_buff_trans: (DynamicBuffer<'a, TriangleVertex>, VertexArrayObject<'a>),

    ell_program: SimpleEllipseProgram<'a>,
    ell_buff: (DynamicBuffer<'a, EllipseVertex>, VertexArrayObject<'a>),
    ell_buff_trans: (DynamicBuffer<'a, EllipseVertex>, VertexArrayObject<'a>),
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
        fn create_buff<'b, V>(res: &'b Renderer) -> (DynamicBuffer<'b, V>, VertexArrayObject<'b>) 
        where 
            V: StaticVertexLayout + 'static
        {
            let vbo = DynamicBuffer::new(res, FORMS_PER_RENDER);
            let vao = VertexArrayObject::new(res, &[&vbo]);
            (vbo, vao)
        }

        Self {
            rec_program: SimpleRectangleProgram::init(renderer).unwrap(),
            rec_buff: create_buff(renderer),
            rec_buff_trans: create_buff(renderer),

            tri_program: SimpleTriangleProgram::init(renderer).unwrap(),
            tri_buff: create_buff(renderer),
            tri_buff_trans: create_buff(renderer),

            ell_program: SimpleEllipseProgram::init(renderer).unwrap(),
            ell_buff: create_buff(renderer),
            ell_buff_trans: create_buff(renderer),
        }
    }

    pub fn draw(&mut self, renderer: &'a Renderer, proj_matrix: &glm::Mat4) {
        self.rec_program.bind();
        self.rec_program.set_u_vp(proj_matrix);
        let (vbo, vao) = &mut self.rec_buff;
        vao.draw_instanced(4, vbo.get_len() as i32, DrawMode::TriangleStrip);

        self.tri_program.bind();
        self.tri_program.set_u_vp(proj_matrix);
        let (vbo, vao) = &mut self.tri_buff;
        vao.draw_instanced(3, vbo.get_len() as i32, DrawMode::Triangles);

        renderer.depth_mask(false);
        self.ell_program.bind();
        self.ell_program.set_u_vp(proj_matrix);
        let (vbo, vao) = &mut self.ell_buff;
        vao.draw_instanced(4, vbo.get_len() as i32, DrawMode::TriangleStrip);
        renderer.depth_mask(true);
    }

    pub fn draw_transparent(&mut self, renderer: &'a Renderer, proj_matrix: &glm::Mat4) {
        renderer.depth_mask(false);

        self.rec_program.bind();
        self.rec_program.set_u_vp(proj_matrix);
        let (vbo, vao) = &mut self.rec_buff_trans;
        vao.draw_instanced(4, vbo.get_len() as i32, DrawMode::TriangleStrip);

        self.tri_program.bind();
        self.tri_program.set_u_vp(proj_matrix);
        let (vbo, vao) = &mut self.tri_buff_trans;
        vao.draw_instanced(3, vbo.get_len() as i32, DrawMode::Triangles);

        self.ell_program.bind();
        self.ell_program.set_u_vp(proj_matrix);
        let (vbo, vao) = &mut self.ell_buff_trans;
        vao.draw_instanced(4, vbo.get_len() as i32, DrawMode::TriangleStrip);
        
        renderer.depth_mask(true);
    }

    fn get_rec_buffer(&mut self, alpha: u8) -> &mut DynamicBuffer<'a, RectangleVertex> {
        if alpha == 255 { &mut self.rec_buff.0 } else { &mut self.rec_buff_trans.0 }
    }

    fn get_tri_buffer(&mut self, alpha: u8) -> &mut DynamicBuffer<'a, TriangleVertex> {
        if alpha == 255 { &mut self.tri_buff.0 } else { &mut self.tri_buff_trans.0 }
    }

    fn get_ell_buffer(&mut self, alpha: u8) -> &mut DynamicBuffer<'a, EllipseVertex> {
        if alpha == 255 { &mut self.ell_buff.0 } else { &mut self.ell_buff_trans.0 }
    }

    pub fn reset(&mut self) {
        self.rec_buff.0.reset_index();
        self.rec_buff_trans.0.reset_index();
        self.tri_buff.0.reset_index();
        self.tri_buff_trans.0.reset_index();
        self.ell_buff.0.reset_index();
        self.ell_buff_trans.0.reset_index();
    }

    pub fn finish(&self) {
        self.rec_buff.0.flush();
        self.rec_buff_trans.0.flush();
        self.tri_buff.0.flush();
        self.tri_buff_trans.0.flush();
        self.ell_buff.0.flush();
        self.ell_buff_trans.0.flush();
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
        buffer.write(&RectangleVertex {
            top_left: oriented_box.top_left,
            top_right: oriented_box.top_right,
            bottom_right: oriented_box.bot_right,
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
        buffer.write(&RectangleVertex {
            top_left: bbox.get_top_left(),
            top_right: bbox.get_top_right(),
            bottom_right: bbox.get_bottom_right(),
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
        buffer.write(&TriangleVertex {
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
        buffer.write(&TriangleVertex {
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
        buffer.write(&EllipseVertex {
            top_left: oriented_box.top_left,
            top_right: oriented_box.top_right,
            bottom_right: oriented_box.bot_right,
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
        buffer.write(&EllipseVertex {
            top_left: simple_box.get_top_left(),
            top_right: simple_box.get_top_right(),
            bottom_right: simple_box.get_bottom_right(),
            color, depth,
        });
    }
}
