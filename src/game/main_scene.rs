use std::{io::{self, BufRead}, time};
use nalgebra_glm as glm;
use crate::{game::{colors::COLOR_DATA, graph::{Graph, GraphBuilder}, instruction::{Instruction, RegisterAddress}, robot::Robot, scene::Scene}, graphics::renderer::{Renderer, fonts::TextRenderConfig, positioning::PositionMode}};

const INSTRUCTION_PERIOD: f32 = 0.25;

pub struct MainScene<'a> {
    robot: Robot,
    graph: Graph<'a>,
    last_instruction_time_passed: f32,
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
                    glm::vec2(0.0, 0.7),
                    glm::vec2(0.6, 0.35),
                    glm::vec2(0.0, -0.2),
                    glm::vec2(1.0, 1.05)
                )
                .add_y_curve(glm::make_vec4(&COLOR_DATA[0]), "Acceleration")
                .add_y_curve(glm::make_vec4(&COLOR_DATA[1]), "Drag")
                .add_horizontal_line(0.0)
                .into_graph(renderer),
            last_instruction_time_passed: 0.0,
        };
    }
}

impl<'a> Scene for MainScene<'a> {
    fn game_logic(&mut self, context: &mut crate::MainContext) {
        if context.halt {
            return;
        }

        self.robot.movement_logic(context.delta, &mut self.graph);

        self.last_instruction_time_passed += context.delta;
        if self.last_instruction_time_passed >= INSTRUCTION_PERIOD {
            self.robot.run_instruction();
            self.last_instruction_time_passed -= INSTRUCTION_PERIOD;
        }
    }

    fn generate_output(&mut self, context: &mut crate::MainContext) {
        let simple_shape_renderer = &mut context.simple_shape_renderer;
        self.robot.draw(simple_shape_renderer);

        context.graph_renderer.draw(
            &self.graph,
            &context.proj_matrix,
            simple_shape_renderer,
            context.renderer,
            context.font,
            1.0,
        );

        simple_shape_renderer.send_ellipse(
            glm::vec2(-1.0, 0.0),
            glm::vec2(0.2, 0.2),
            PositionMode::Center,
            glm::make_vec4(&COLOR_DATA[1]),
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
            glm::make_vec4(&COLOR_DATA[3]),
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
