use std::time;
use nalgebra_glm as glm;
use crate::{game::{graph::{Graph, GraphBuilder}, instruction::{Instruction, RegisterAddress}, robot::Robot, scene::Scene}, graphics::renderer::{Renderer, fonts::TextRenderConfig, positioning::PositionMode}};

const INSTRUCTION_PERIOD: f32 = 0.25;

pub struct MainScene<'a> {
    robot: Robot,
    graph: Graph<'a>,
    last_instruction_execution: time::Instant,
    time_start: time::Instant,
}

impl<'a> MainScene<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        let x0 = RegisterAddress::new(0);
        let x2 = RegisterAddress::new(2);
        return Self { 
            robot: Robot::new()
                .set_instruction(0, Instruction::new_lui(x2, 10))
                .set_instruction(1, Instruction::new_sys(3))
                .set_instruction(2, Instruction::new_subi(x2, x2, 1))
                .set_instruction(3, Instruction::new_bne(x2, x0, 14))

                .set_instruction(4, Instruction::new_sys(1))

                .set_instruction(5, Instruction::new_lui(x2, 10))
                .set_instruction(6, Instruction::new_sys(4))
                .set_instruction(7, Instruction::new_subi(x2, x2, 1))
                .set_instruction(8, Instruction::new_bne(x2, x0, 14))

                .set_instruction(9, Instruction::new_sys(2)),
            graph: GraphBuilder::new(
                    glm::vec2(0.0, 0.6),
                    glm::vec2(0.6, 0.6),
                    glm::vec2(0.0, 0.0),
                    glm::vec2(1.0, 1.0)
                )
                .set_func(|x| 3.0 * x * x - 2.0 * x * x * x)
                .add_horizontal_line(1.0)
                .add_vertical_line(1.0)
                .into_graph(renderer),
            last_instruction_execution: time::Instant::now(),
            time_start: time::Instant::now(),
        };
    }
}

impl<'a> Scene for MainScene<'a> {
    fn game_logic(&mut self, context: &mut crate::MainContext) {
        // let time = context.now.duration_since(self.time_start).as_secs_f32();
        // let x = (((time % 12.0) - 0.5) / 11.0).clamp(0.0, 1.0);
        self.robot.movement_logic(context.delta);

        if context.now.duration_since(self.last_instruction_execution).as_secs_f32() >= INSTRUCTION_PERIOD {
            self.robot.run_instruction();
            self.last_instruction_execution = context.now;
        }
    }

    fn generate_output(&mut self, context: &mut crate::MainContext) {
        let simple_shape_renderer = &mut context.simple_shape_renderer;
        self.robot.draw(simple_shape_renderer);

        // simple_shape_renderer.send_rectangle(
        //     glm::vec2(0.75, 0.0),
        //     glm::vec2(0.1, 1.2),
        //     PositionMode::Center,
        //     glm::vec4(200, 0, 0, 255),
        //     100, 0);
        // self.robot.draw(simple_shape_renderer);

        // let time = context.now.duration_since(self.time_start).as_secs_f32();
        // let x = (((time % 12.0) - 0.5) / 11.0).clamp(0.0, 1.0);

        // context.graph_renderer.draw(
        //     &self.graph,
        //     &context.proj_matrix,
        //     simple_shape_renderer,
        //     context.font,
        //     x,
        // );

        simple_shape_renderer.send_ellipse(
            glm::vec2(-1.0, 0.0),
            glm::vec2(0.2, 0.2),
            PositionMode::Center,
            glm::vec4(255, 165, 90, 255),
            2, 0,
        );

        // context.font.add_text(&TextRenderConfig::new(
        //     "Origin",
        //     glm::vec2(-1.0, -0.2),
        //     0.1, glm::vec3(255, 165, 90),
        // ).set_mode(PositionMode::TopCenter));

        simple_shape_renderer.send_ellipse(
            glm::vec2(1.0, 0.0),
            glm::vec2(0.2, 0.2),
            PositionMode::Center,
            glm::vec4(220, 230, 240, 255),
            2, 0,
        );

        // context.font.add_text(&TextRenderConfig::new(
        //     "Target",
        //     glm::vec2(1.0, -0.2),
        //     0.1, glm::vec3(220, 230, 240),
        // ).set_mode(PositionMode::TopCenter));
    }

    fn handle_input(&mut self, context: &mut crate::MainContext) {
        
    }
}
