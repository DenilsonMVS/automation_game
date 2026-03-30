
use macros::program_interface;
use nalgebra_glm as glm;

use crate::graphics::renderer::{Renderer, buffer::{GenericBuffer, StaticBuffer}, drawable::DrawMode, fonts::{Font, TextRenderConfig}, positioning::PositionMode, program::{Program, ShaderType}, simple_shape_renderer::SimpleShapeRenderer, uniform::UniformValue, vertex_array_object::VertexArrayObject};
use anyhow::Result;

const Y_BUFFER_SIZE: usize = 1 << 7;

#[repr(C)]
#[derive(Debug)]
struct GraphSSBO {
    center: glm::Vec2,
    dimensions: glm::Vec2,
    view_min: glm::Vec2,
    view_max: glm::Vec2,
    color: glm::U8Vec4,
    thickness: f32,
    bellow_alpha: f32,
    y: [f32; Y_BUFFER_SIZE],
}

#[program_interface(
	vert = "../../res/shaders/graph.vert",
	frag = "../../res/shaders/graph.frag"
)]
struct GraphProgram {
    u_projection: glm::Mat4,
    u_display_proportion: f32,
}

pub struct GraphRenderer<'a> {
    program: GraphProgram<'a>,
    vao: VertexArrayObject<'a>,
}

impl<'a> GraphRenderer<'a> {
    pub fn new(renderer: &'a Renderer) -> Result<Self> {
        let program = GraphProgram::init(renderer).unwrap();
        let vao = VertexArrayObject::new(renderer, &[]);
        return Ok(Self { program, vao });
    }

    pub fn draw(&self,
        graph: &Graph,
        proj_matrix: &glm::Mat4,
        simple_shape_renderer: &mut SimpleShapeRenderer,
        font: &mut Font,
        display_proportion: f32,
    ) {
        self.program.bind();
        self.program.set_u_display_proportion(&display_proportion);
        self.program.set_u_projection(proj_matrix);
        graph.buffer.bind_as_storage(0);
        self.vao.draw(4, DrawMode::TriangleFan);

        let min_pos = graph.center - graph.dimensions * 0.5;
        simple_shape_renderer.send_rectangle(
            min_pos - glm::vec2(0.0025, 0.0025),
            glm::vec2(0.005, graph.dimensions.y),
            PositionMode::BottomLeft,
            graph.secondary_color,
            3000, 1
        );
        simple_shape_renderer.send_rectangle(
            min_pos - glm::vec2(0.0025, 0.0025),
            glm::vec2(graph.dimensions.x, 0.005),
            PositionMode::BottomLeft,
            graph.secondary_color,
            3000, 1
        );

        for vertical_line in graph.vertical_lines.iter() {
            if graph.view_min.x < *vertical_line && *vertical_line < graph.view_max.x {
                let mix_pos = (*vertical_line - graph.view_min.x) / (graph.view_max.x - graph.view_min.x);
                let pos = mix_pos * graph.dimensions.x;
                simple_shape_renderer.send_rectangle(
                    min_pos + glm::vec2(pos, 0.0),
                    glm::vec2(0.0025, graph.dimensions.y),
                    PositionMode::BottomCenter,
                    graph.secondary_color - glm::vec4(0, 0, 0, 100),
                    3000, 1
                );

                font.add_text(&TextRenderConfig {
                    text: &format!("{:.1}", *vertical_line),
                    position: min_pos + glm::vec2(pos, -0.01),
                    position_mode: PositionMode::TopCenter,
                    line_height: 0.04,
                    rotation: 0.0,
                    color: glm::vec3(graph.secondary_color.x, graph.secondary_color.y, graph.secondary_color.z),
                    group: 1,
                });
            } 
        }

        for horizontal_line in graph.horizontal_lines.iter() {
            if graph.view_min.y < *horizontal_line && *horizontal_line < graph.view_max.y {
                let mix_pos = (*horizontal_line - graph.view_min.y) / (graph.view_max.y - graph.view_min.y);
                let pos = mix_pos * graph.dimensions.y;
                simple_shape_renderer.send_rectangle(
                    min_pos + glm::vec2(0.0, pos),
                    glm::vec2(graph.dimensions.x, 0.0025),
                    PositionMode::CenterLeft,
                    graph.secondary_color - glm::vec4(0, 0, 0, 100),
                    3000, 1
                );

                font.add_text(&TextRenderConfig {
                    text: &format!("{:.1}", *horizontal_line),
                    position: min_pos + glm::vec2(-0.01, pos),
                    position_mode: PositionMode::CenterRight,
                    line_height: 0.04,
                    rotation: 0.0,
                    color: glm::vec3(graph.secondary_color.x, graph.secondary_color.y, graph.secondary_color.z),
                    group: 1,
                });
            }
        }

        simple_shape_renderer.send_symmetrical_triangle(
            min_pos + glm::vec2(0.0, graph.dimensions.y + 0.025),
            min_pos + glm::vec2(0.0, graph.dimensions.y),
            0.025,
            graph.secondary_color,
            3000, 1
        );
        simple_shape_renderer.send_symmetrical_triangle(
            min_pos + glm::vec2(graph.dimensions.x + 0.025, 0.0),
            min_pos + glm::vec2(graph.dimensions.x, 0.0),
            0.025,
            graph.secondary_color,
            3000, 1
        );
    }
}

pub struct GraphBuilder {
    center: glm::Vec2,
    dimensions: glm::Vec2,
    view_min: glm::Vec2,
    view_max: glm::Vec2,
    vertical_lines: Vec<f32>,
    horizontal_lines: Vec<f32>,
    primary_color: glm::U8Vec4,
    secondary_color: glm::U8Vec4,
    thickness: f32,
    bellow_alpha: f32,
    y_values: [f32; Y_BUFFER_SIZE],
}

impl GraphBuilder {
    pub fn new(
        center: glm::Vec2, dimensions: glm::Vec2,
        view_min: glm::Vec2, view_max: glm::Vec2,
    ) -> Self {
        Self {
            center, dimensions,
            view_min, view_max,
            vertical_lines: Vec::new(),
            horizontal_lines: Vec::new(),
            primary_color: glm::vec4(0, 229, 255, 255),
            secondary_color: glm::vec4(150, 150, 150, 255),
            thickness: 0.01, bellow_alpha: 0.01,
            y_values: [0.0; Y_BUFFER_SIZE],
        }
    }

    pub fn set_primary_color(mut self, primary_color: glm::U8Vec4) -> Self {
        self.primary_color = primary_color;
        return self;
    }

    pub fn set_secondary_color(mut self, secondary_color: glm::U8Vec4) -> Self {
        self.secondary_color = secondary_color;
        return self;
    }

    pub fn set_thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        return self;
    }

    pub fn set_bellow_alpha(mut self, bellow_alpha: f32) -> Self {
        self.bellow_alpha = bellow_alpha;
        return self;
    }

    pub fn set_y_values(mut self, y_values: &[f32; Y_BUFFER_SIZE]) -> Self {
        self.y_values = *y_values;
        return self;
    }

    pub fn set_func<F>(mut self, func: F) -> Self
    where
        F: Fn(f32) -> f32
    {
        let x_range = self.view_max.x - self.view_min.x;
        let x_step = x_range / (Y_BUFFER_SIZE - 1) as f32;
        let y_values: [f32; Y_BUFFER_SIZE] = std::array::from_fn(|i| {
            let x = self.view_min.x + i as f32 * x_step;
            return func(x);
        });

        self.y_values = y_values;
        return self;
    }

    pub fn add_horizontal_line(mut self, line: f32) -> Self {
        self.horizontal_lines.push(line);
        return self;
    }

    pub fn add_vertical_line(mut self, line: f32) -> Self {
        self.vertical_lines.push(line);
        return self;
    }

    pub fn into_graph<'a>(&self, renderer: &'a Renderer) -> Graph<'a> {
        let ssbo_data = GraphSSBO {
            center: self.center,
            dimensions: self.dimensions,
            color: self.primary_color,
            thickness: self.thickness,
            bellow_alpha: self.thickness,
            view_min: self.view_min,
            view_max: self.view_max,
            y: self.y_values,
        };

        let buffer = StaticBuffer::new(renderer, &[ssbo_data]);
        return Graph {
            center: self.center,
            dimensions: self.dimensions,
            view_min: self.view_min,
            view_max: self.view_max,
            secondary_color: self.secondary_color,
            buffer,
            horizontal_lines: vec![],
            vertical_lines: vec![],
        };
    }
}

pub struct Graph<'a> {
    center: glm::Vec2,
    dimensions: glm::Vec2,
    view_min: glm::Vec2,
    view_max: glm::Vec2,
    secondary_color: glm::U8Vec4,
    buffer: StaticBuffer<'a, GraphSSBO>,
    vertical_lines: Vec<f32>,
    horizontal_lines: Vec<f32>,
}
