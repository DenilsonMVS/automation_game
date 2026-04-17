
use macros::program_interface;
use nalgebra_glm as glm;

use crate::graphics::renderer::{Capability, Renderer, buffer::{DynamicUnitBuffer, GenericBuffer}, drawable::DrawMode, fonts::{Font, TextRenderConfig}, positioning::PositionMode, program::{Program, ShaderType}, simple_shape_renderer::SimpleShapeRenderer, uniform::UniformValue, vertex_array_object::VertexArrayObject};
use anyhow::Result;

const LINE_HEIGHT: f32 = 0.04;
const LINE_SPACING: f32 = LINE_HEIGHT + 0.01;
const Y_BUFFER_SIZE: usize = 1 << 10;

#[repr(C)]
#[derive(Debug)]
pub struct GraphSSBO {
    center: glm::Vec2,
    dimensions: glm::Vec2,
    view_min: glm::Vec2,
    view_max: glm::Vec2,
    color: glm::U8Vec4,
    thickness: f32,
    bellow_alpha: f32,
    buf_writen: u32,
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
        renderer: &Renderer,
        font: &mut Font,
        display_proportion: f32,
    ) {
        self.program.bind();
        self.program.set_u_display_proportion(&display_proportion);
        self.program.set_u_projection(proj_matrix);

        renderer.disable(Capability::DepthTest);
        for (_, buffer, _) in graph.buffers.iter() {
            buffer.bind_as_storage(0);
            self.vao.draw(4, DrawMode::TriangleFan);
        }

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
                    line_height: LINE_HEIGHT,
                    rotation: 0.0,
                    color: graph.secondary_color,
                    group: 1,
                    depth: 2000,
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
                    line_height: LINE_HEIGHT,
                    rotation: 0.0,
                    color: graph.secondary_color,
                    group: 1,
                    depth: 2000,
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

        let top_graph = graph.center + glm::vec2(-graph.dimensions.x * 1.4, graph.dimensions.y) * 0.5;
        for (i, (_, _, y_val)) in graph.buffers.iter().enumerate() {
            let position = top_graph - glm::vec2(0.0, LINE_SPACING * i as f32);
            font.add_text(
                &TextRenderConfig::new(
                    y_val.label,
                    position,
                    LINE_HEIGHT,
                    y_val.color,
                )
                .set_mode(PositionMode::CenterRight)
                .set_group(1),
            );

            simple_shape_renderer.send_ellipse(
                position + glm::vec2(0.01, 0.0),
                glm::vec2(0.025, 0.025),
                PositionMode::CenterLeft,
                y_val.color, 4000, 1,
            );
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct YValue {
    color: glm::U8Vec4,
    label: &'static str,
}

pub struct GraphBuilder {
    center: glm::Vec2,
    dimensions: glm::Vec2,
    view_min: glm::Vec2,
    view_max: glm::Vec2,
    vertical_lines: Vec<f32>,
    horizontal_lines: Vec<f32>,
    secondary_color: glm::U8Vec4,
    thickness: f32,
    bellow_alpha: f32,
    y_values: Vec<YValue>,
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
            secondary_color: glm::vec4(150, 150, 150, 255),
            thickness: 0.02, bellow_alpha: 0.01,
            y_values: Vec::new(),
        }
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

    pub fn add_horizontal_line(mut self, line: f32) -> Self {
        self.horizontal_lines.push(line);
        return self;
    }

    pub fn add_vertical_line(mut self, line: f32) -> Self {
        self.vertical_lines.push(line);
        return self;
    }

    pub fn add_y_curve(mut self, color: glm::U8Vec4, label: &'static str) -> Self {
        self.y_values.push(YValue { color, label });
        return self;
    }

    pub fn into_graph<'a>(self, renderer: &'a Renderer) -> Graph<'a> {
        let buffers = self.y_values.iter().map(|y_val| (0, DynamicUnitBuffer::new(renderer, &[GraphSSBO {
            center: self.center,
            dimensions: self.dimensions,
            view_min: self.view_min,
            view_max: self.view_max,
            color: y_val.color,
            thickness: self.thickness,
            bellow_alpha: self.bellow_alpha,
            buf_writen: 0,
            y: [0.0; Y_BUFFER_SIZE],
        }]), *y_val)).collect();

        return Graph {
            center: self.center,
            dimensions: self.dimensions,
            view_min: self.view_min,
            view_max: self.view_max,
            secondary_color: self.secondary_color,
            buffers,
            horizontal_lines: self.horizontal_lines,
            vertical_lines: self.vertical_lines,
        };
    }
}

pub struct Graph<'a> {
    center: glm::Vec2,
    dimensions: glm::Vec2,
    view_min: glm::Vec2,
    view_max: glm::Vec2,
    secondary_color: glm::U8Vec4,
    buffers: Vec<(usize, DynamicUnitBuffer<'a, GraphSSBO>, YValue)>,
    vertical_lines: Vec<f32>,
    horizontal_lines: Vec<f32>,
}

impl<'a> Graph<'a> {
    pub fn push(&mut self, buffer_idx: usize, val: f32) {
        let (ref mut count, ref mut unit_buffer, _) = self.buffers[buffer_idx];
        if *count < Y_BUFFER_SIZE {
            let buf = unit_buffer.mut_ref();
            buf.y[*count] = val;
            *count += 1;
            buf.buf_writen = *count as u32;
        }
    }
}

