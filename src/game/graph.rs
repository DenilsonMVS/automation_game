
use macros::{GlVertex, program_interface};
use nalgebra_glm as glm;

use crate::graphics::renderer::{Renderer, buffer::{GenericBuffer, StaticBuffer}, drawable::DrawMode, positioning::PositionMode, program::{Program, ShaderType}, simple_shape_renderer::SimpleShapeRenderer, uniform::UniformValue, vertex_array_object::{FieldType, StaticVertexLayout, VertexArrayObject}};

const Y_BUFFER_SIZE: usize = 1 << 8;

#[repr(C)]
#[derive(Debug, GlVertex)]
struct YValue {
    y: f32,
}

#[program_interface(
	vert = "../../res/shaders/graph.vert",
	frag = "../../res/shaders/graph.frag"
)]
struct GraphProgram {
    u_center: glm::Vec2,
    u_dimensions: glm::Vec2,
    u_projection: glm::Mat4,
    u_color: glm::Vec4,
    u_view_min: glm::Vec2,
    u_view_max: glm::Vec2,
}

pub struct Graph<'a> {
    center: glm::Vec2,
    dimensions: glm::Vec2,
    view_min: glm::Vec2,
    view_max: glm::Vec2,
    primary_color: glm::U8Vec4,
    secondary_color: glm::U8Vec4,
    y_buffer: StaticBuffer<'a, YValue>,
    program: GraphProgram<'a>,
    vao: VertexArrayObject<'a>,
}

impl<'a> Graph<'a> {
    pub fn new(
        renderer: &'a Renderer,
        center: glm::Vec2, dimensions: glm::Vec2,
        view_min: glm::Vec2, view_max: glm::Vec2,
        primary_color: glm::U8Vec4, secondary_color: glm::U8Vec4,
    ) -> Self {
        let program = GraphProgram::init(renderer).unwrap();
        program.set_u_center(&center);
        program.set_u_dimensions(&dimensions);
        program.set_u_color(&primary_color.map(|x| (x as f32) / 255.0));
        program.set_u_center(&center);
        program.set_u_view_min(&view_min);
        program.set_u_view_max(&view_max);

        let x_range = view_max.x - view_min.x;
        let x_step = x_range / Y_BUFFER_SIZE as f32;
        let y_data: [_; Y_BUFFER_SIZE] = std::array::from_fn(|i| {
            let x = view_min.x + i as f32 * x_step;
            let y = (10.0 * x).sin() * 0.4 + 0.5;
            return YValue { y };
        });

        let y_buffer = StaticBuffer::new(renderer, &y_data);
        y_buffer.bind_as_storage(0);

        return Self {
            center, dimensions,
            view_min, view_max,
            primary_color, secondary_color,
            y_buffer,
            program,
            vao: VertexArrayObject::new(renderer, &[]),
        };
    }

    pub fn draw(&self, proj_matrix: &glm::Mat4, simple_shape_renderer: &mut SimpleShapeRenderer) {
        self.program.bind();
        self.program.set_u_projection(proj_matrix);
        self.vao.draw(4, DrawMode::TriangleFan);

        let min_pos = self.center - self.dimensions * 0.5;
        simple_shape_renderer.send_rectangle(
            min_pos - glm::vec2(0.0025, 0.0025),
            glm::vec2(0.005, self.dimensions.y),
            PositionMode::BottomLeft,
            self.secondary_color,
            3000,
        );
        simple_shape_renderer.send_rectangle(
            min_pos - glm::vec2(0.0025, 0.0025),
            glm::vec2(self.dimensions.x, 0.005),
            PositionMode::BottomLeft,
            self.secondary_color,
            3000,
        );

        simple_shape_renderer.send_symmetrical_triangle(
            min_pos + glm::vec2(0.0, self.dimensions.y + 0.025),
            min_pos + glm::vec2(0.0, self.dimensions.y),
            0.025,
            self.secondary_color,
            3000
        );
        simple_shape_renderer.send_symmetrical_triangle(
            min_pos + glm::vec2(self.dimensions.x + 0.025, 0.0),
            min_pos + glm::vec2(self.dimensions.x, 0.0),
            0.025,
            self.secondary_color,
            3000
        );
    }
}