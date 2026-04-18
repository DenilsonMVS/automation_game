use macros::{GlVertex, program_interface};
use nalgebra_glm as glm;

use crate::graphics::renderer::{Renderer, buffer::DynamicBuffer, drawable::DrawMode, positioning::{OrientedBox, PositionMode, RenderBox}, program::{Program, ShaderType}, uniform::UniformValue, vertex_array_object::{FieldType, StaticVertexLayout, VertexArrayObject}};

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
    rec_buffs: [(DynamicBuffer<'a, RectangleVertex>, VertexArrayObject<'a>); 2],

    tri_program: SimpleTriangleProgram<'a>,
    tri_buffs: [(DynamicBuffer<'a, TriangleVertex>, VertexArrayObject<'a>); 2],

    ell_program: SimpleEllipseProgram<'a>,
    ell_buffs: [(DynamicBuffer<'a, EllipseVertex>, VertexArrayObject<'a>); 2],
}

impl<'a> SimpleShapeRenderer<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        let rec_program = SimpleRectangleProgram::init(renderer).unwrap();
        let rec_buffs = std::array::from_fn(|_| {
            let vbo = DynamicBuffer::new(renderer, FORMS_PER_RENDER);
            let vao = VertexArrayObject::new(renderer, &[&vbo]);
            return (vbo, vao);
        });

        let tri_program = SimpleTriangleProgram::init(renderer).unwrap();
        let tri_buffs = std::array::from_fn(|_| {
            let vbo = DynamicBuffer::new(renderer, FORMS_PER_RENDER);
            let vao = VertexArrayObject::new(renderer, &[&vbo]);
            return (vbo, vao);
        });

        let ell_program = SimpleEllipseProgram::init(renderer).unwrap();
        let ell_buffs = std::array::from_fn(|_| {
            let vbo = DynamicBuffer::new(renderer, FORMS_PER_RENDER);
            let vao = VertexArrayObject::new(renderer, &[&vbo]);
            return (vbo, vao);
        });
        
        return Self {
            rec_program, rec_buffs,
            tri_program, tri_buffs,
            ell_program, ell_buffs,
        };
    }

    pub fn draw(&mut self, renderer: &'a Renderer, proj_matrix: &glm::Mat4, vp_matrix: &glm::Mat4) {
        self.rec_program.bind();
        for (i, (vbo, vao)) in self.rec_buffs.iter_mut().enumerate() {
            if i == 0 {
                self.rec_program.set_u_vp(vp_matrix);
            } else {
                self.rec_program.set_u_vp(proj_matrix);
            }

            vao.draw_instanced(4, vbo.get_len() as i32, DrawMode::TriangleStrip);
        }

        self.tri_program.bind();
        for (i, (vbo, vao)) in self.tri_buffs.iter_mut().enumerate() {
            if i == 0 {
                self.tri_program.set_u_vp(vp_matrix);
            } else {
                self.tri_program.set_u_vp(proj_matrix);
            }

            vao.draw_instanced(3, vbo.get_len() as i32, DrawMode::Triangles);
        }

        renderer.depth_mask(false);
        self.ell_program.bind();
        for (i, (vbo, vao)) in self.ell_buffs.iter_mut().enumerate() {
            if i == 0 {
                self.ell_program.set_u_vp(vp_matrix);
            } else {
                self.ell_program.set_u_vp(proj_matrix);
            }

            vao.draw_instanced(4, vbo.get_len() as i32, DrawMode::TriangleStrip);
        }
        renderer.depth_mask(true);
    }

    pub fn reset(&mut self) {
        for (vbo, _) in self.rec_buffs.iter_mut() {
            vbo.reset_index();
        }
        for (vbo, _) in self.tri_buffs.iter_mut() {
            vbo.reset_index();
        }
        for (vbo, _) in self.ell_buffs.iter_mut() {
            vbo.reset_index();
        }
    }

    pub fn finish(&self) {
        for (vbo, _) in self.rec_buffs.iter() {
            vbo.flush();
        }
        for (vbo, _) in self.tri_buffs.iter() {
            vbo.flush();
        }
        for (vbo, _) in self.ell_buffs.iter() {
            vbo.flush();
        }
    }

    pub fn send_rotated_rectangle(
        &mut self,
        position: glm::Vec2,
        size: glm::Vec2,
        position_mode: PositionMode,
        up_vector: glm::Vec2,
        color: glm::U8Vec4,
        depth: u16,
        group: usize,
    ) {
        let oriented_box = OrientedBox::new_rotated(position, position_mode, size, up_vector);
        self.rec_buffs[group].0.write(&RectangleVertex {
            top_left: oriented_box.top_left,
            top_right: oriented_box.top_right,
            bottom_right: oriented_box.bot_right,
            color, depth,
        });
    }

    pub fn preview_render_box(
        &self,
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
        group: usize,
    ) {
        self.rec_buffs[group].0.write(&RectangleVertex {
            top_left: simple_box.get_top_left(),
            top_right: simple_box.get_top_right(),
            bottom_right: simple_box.get_bottom_right(),
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
        group: usize,
    ) -> RenderBox {
        let simple_box = RenderBox::new_simple(position, position_mode, size);
        self.send_render_box(&simple_box, color, depth, group);
        return simple_box;
    }

    pub fn send_symmetrical_triangle(
        &mut self,
        apex: glm::Vec2,
        centroid: glm::Vec2,
        width: f32,
        color: glm::U8Vec4,
        depth: u16,
        group: usize,
    ) {
        let to_centroid = centroid - apex;
        let dist_r = glm::length(&to_centroid);

        let unit_h = to_centroid / dist_r;
        let unit_w = glm::vec2(-unit_h.y, unit_h.x);

        let base_midpoint = apex + (unit_h * (dist_r * 1.5));
        let half_width = width * 0.5;
        let vertex_b = base_midpoint + (unit_w * half_width);
        let vertex_c = base_midpoint - (unit_w * half_width);

        self.tri_buffs[group].0.write(&TriangleVertex {
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
        group: usize,
    ) {
        self.tri_buffs[group].0.write(&TriangleVertex {
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
        group: usize,
    ) {
        let oriented_box = OrientedBox::new_rotated(position, position_mode, size, up_vector);
        self.ell_buffs[group].0.write(&EllipseVertex {
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
        group: usize,
    ) {
        let simple_box = RenderBox::new_simple(position, position_mode, size);
        self.ell_buffs[group].0.write(&EllipseVertex {
            top_left: simple_box.get_top_left(),
            top_right: simple_box.get_top_right(),
            bottom_right: simple_box.get_bottom_right(),
            color, depth,
        });
    }
}
