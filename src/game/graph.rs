
use macros::program_interface;
use nalgebra_glm as glm;

use crate::graphics::renderer::{Renderer, buffer::{GenericBuffer, StaticBuffer}, drawable::DrawMode, positioning::PositionMode, program::{Program, ShaderType}, simple_shape_renderer::SimpleShapeRenderer, uniform::UniformValue, vertex_array_object::VertexArrayObject};

const Y_BUFFER_SIZE: usize = 1 << 7;

#[repr(C)]
#[derive(Debug)]
struct GraphSSBO {
    center: glm::Vec2,
    dimensions: glm::Vec2,
    color: glm::Vec4,
    view_min: glm::Vec2,
    view_max: glm::Vec2,
    y: [f32; Y_BUFFER_SIZE],
}

#[program_interface(
	vert = "../../res/shaders/graph.vert",
	frag = "../../res/shaders/graph.frag"
)]
struct GraphProgram {
    u_projection: glm::Mat4,
}

pub struct Graph<'a> {
    center: glm::Vec2,
    dimensions: glm::Vec2,
    view_min: glm::Vec2,
    view_max: glm::Vec2,
    secondary_color: glm::U8Vec4,
    buffer: StaticBuffer<'a, GraphSSBO>,
    program: GraphProgram<'a>,
    vao: VertexArrayObject<'a>,
    vertical_lines: Vec<f32>,
    horizontal_lines: Vec<f32>,
}

impl<'a> Graph<'a> {
    pub fn new(
        renderer: &'a Renderer,
        center: glm::Vec2, dimensions: glm::Vec2,
        view_min: glm::Vec2, view_max: glm::Vec2,
        primary_color: glm::U8Vec4, secondary_color: glm::U8Vec4,
    ) -> Self {
        let program = GraphProgram::init(renderer).unwrap();

        let x_range = view_max.x - view_min.x;
        let x_step = x_range / (Y_BUFFER_SIZE - 1) as f32;
        let y_data: [f32; Y_BUFFER_SIZE] = std::array::from_fn(|i| {
            let x = view_min.x + i as f32 * x_step;
            (10.0 * x).sin() * 0.7
        });

        let ssbo_data = GraphSSBO {
            center,
            dimensions,
            color: primary_color.map(|x| (x as f32) / 255.0),
            view_min,
            view_max,
            y: y_data,
        };

        let buffer = StaticBuffer::new(renderer, &[ssbo_data]);

        return Self {
            center, dimensions,
            view_min, view_max,
            secondary_color,
            buffer,
            program,
            vao: VertexArrayObject::new(renderer, &[]),
            horizontal_lines: vec![0.0],
            vertical_lines: vec![0.0],
        };
    }

    pub fn draw(&self, proj_matrix: &glm::Mat4, simple_shape_renderer: &mut SimpleShapeRenderer) {
        self.program.bind();
        self.program.set_u_projection(proj_matrix);
        self.buffer.bind_as_storage(0);
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

        for vertical_line in self.vertical_lines.iter() {
            if self.view_min.x < *vertical_line && *vertical_line < self.view_max.x {
                let mix_pos = (*vertical_line - self.view_min.x) / (self.view_max.x - self.view_min.x);
                let pos = mix_pos * self.dimensions.x;
                simple_shape_renderer.send_rectangle(
                    min_pos + glm::vec2(pos, 0.0),
                    glm::vec2(0.0025, self.dimensions.y),
                    PositionMode::BottomCenter,
                    self.secondary_color - glm::vec4(0, 0, 0, 100),
                    3000,
                );
            } 
        }

        for horizontal_line in self.horizontal_lines.iter() {
            if self.view_min.y < *horizontal_line && *horizontal_line < self.view_max.y {
                let mix_pos = (*horizontal_line - self.view_min.y) / (self.view_max.y - self.view_min.y);
                let pos = mix_pos * self.dimensions.y;
                simple_shape_renderer.send_rectangle(
                    min_pos + glm::vec2(0.0, pos),
                    glm::vec2(self.dimensions.x, 0.0025),
                    PositionMode::CenterLeft,
                    self.secondary_color - glm::vec4(0, 0, 0, 100),
                    3000,
                );
            } 
        }

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