use macros::{GlVertex, program_interface};
use nalgebra_glm as glm;

use crate::graphics::renderer::{Renderer, buffer::DynamicBuffer, drawable::DrawMode, positioning::{OrientedBox, PositionMode, RenderBox}, program::{Program, ShaderType}, uniform::UniformValue, vertex_array_object::{FieldType, StaticVertexLayout, VertexArrayObject}};

const FORMS_PER_RENDER: usize = 1 << 10;

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
    rec_vbo: DynamicBuffer<'a, RectangleVertex>,
    rec_vao: VertexArrayObject<'a>,

    tri_program: SimpleTriangleProgram<'a>,
    tri_vbo: DynamicBuffer<'a, TriangleVertex>,
    tri_vao: VertexArrayObject<'a>,

    ell_program: SimpleEllipseProgram<'a>,
    ell_vbo: DynamicBuffer<'a, EllipseVertex>,
    ell_vao: VertexArrayObject<'a>,
}

impl<'a> SimpleShapeRenderer<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        let rec_program = SimpleRectangleProgram::init(renderer).unwrap();
        let rec_vbo = DynamicBuffer::new(renderer, FORMS_PER_RENDER);
        let rec_vao = VertexArrayObject::new(renderer, &[&rec_vbo]);

        let tri_program = SimpleTriangleProgram::init(renderer).unwrap();
        let tri_vbo = DynamicBuffer::new(renderer, FORMS_PER_RENDER);
        let tri_vao = VertexArrayObject::new(renderer, &[&tri_vbo]);

        let ell_program = SimpleEllipseProgram::init(renderer).unwrap();
        let ell_vbo = DynamicBuffer::new(renderer, FORMS_PER_RENDER);
        let ell_vao = VertexArrayObject::new(renderer, &[&ell_vbo]);
        
        return Self {
            rec_program, rec_vbo, rec_vao,
            tri_program, tri_vbo, tri_vao,
            ell_program, ell_vbo, ell_vao,
        };
    }

    pub fn draw(&mut self, vp_matrix: &glm::Mat4) {
        self.rec_program.bind();
        self.rec_program.set_u_vp(vp_matrix);

        self.rec_vao.draw_instanced(4, self.rec_vbo.get_len() as i32, DrawMode::TriangleStrip);
        self.rec_vbo.lock();

        self.tri_program.bind();
        self.tri_program.set_u_vp(vp_matrix);

        self.tri_vao.draw_instanced(3, self.tri_vbo.get_len() as i32, DrawMode::TriangleStrip);
        self.tri_vbo.lock();

        self.ell_program.bind();
        self.ell_program.set_u_vp(vp_matrix);

        self.ell_vao.draw_instanced(4, self.ell_vbo.get_len() as i32, DrawMode::TriangleStrip);
        self.ell_vbo.lock();
    }

    pub fn reset(&mut self) {
        self.rec_vbo.wait();
        self.tri_vbo.wait();
        self.ell_vbo.wait();

        self.rec_vbo.reset_index();
        self.tri_vbo.reset_index();
        self.ell_vbo.reset_index();
    }

    pub fn finish(&self) {
        self.rec_vbo.flush();
        self.tri_vbo.flush();
        self.ell_vbo.flush();
    }

    pub fn send_rotated_rectangle(
        &mut self,
        position: glm::Vec2,
        size: glm::Vec2,
        position_mode: PositionMode,
        up_vector: glm::Vec2,
        color: glm::U8Vec4,
        depth: u16,
    ) {
        let oriented_box = OrientedBox::new_rotated(position, position_mode, size, up_vector);
        self.rec_vbo.write(&RectangleVertex {
            top_left: oriented_box.top_left,
            top_right: oriented_box.top_right,
            bottom_right: oriented_box.bot_right,
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
    ) {
        let simple_box = RenderBox::new_simple(position, position_mode, size);
        self.rec_vbo.write(&RectangleVertex {
            top_left: simple_box.get_top_left(),
            top_right: simple_box.get_top_right(),
            bottom_right: simple_box.get_bottom_right(),
            color, depth,
        });
    }

    pub fn send_symmetrical_triangle(
        &mut self,
        apex: glm::Vec2,
        centroid: glm::Vec2,
        width: f32,
        color: glm::U8Vec4,
        depth: u16,
    ) {
        let to_centroid = centroid - apex;
        let dist_r = glm::length(&to_centroid);

        let unit_h = to_centroid / dist_r;
        let unit_w = glm::vec2(-unit_h.y, unit_h.x);

        let base_midpoint = apex + (unit_h * (dist_r * 1.5));
        let half_width = width * 0.5;
        let vertex_b = base_midpoint + (unit_w * half_width);
        let vertex_c = base_midpoint - (unit_w * half_width);

        self.tri_vbo.write(&TriangleVertex {
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
    ) {
        self.tri_vbo.write(&TriangleVertex {
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
    ) {
        let oriented_box = OrientedBox::new_rotated(position, position_mode, size, up_vector);
        self.ell_vbo.write(&EllipseVertex {
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
    ) {
        let simple_box = RenderBox::new_simple(position, position_mode, size);
        self.ell_vbo.write(&EllipseVertex {
            top_left: simple_box.get_top_left(),
            top_right: simple_box.get_top_right(),
            bottom_right: simple_box.get_bottom_right(),
            color, depth,
        });
    }
}
