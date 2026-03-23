use std::time;
use nalgebra_glm as glm;
use crate::{game::{graph::Graph, instruction::{Instruction, RegisterAddress}, robot::Robot, scene::Scene}, graphics::renderer::Renderer};

const INSTRUCTION_PERIOD: f32 = 0.25;

pub struct MainScene<'a> {
    robot: Robot,
    graph: Graph<'a>,
    last_instruction_execution: time::Instant,
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
            graph: Graph::new(
                renderer,
                glm::vec2(0.0, 0.0),
                glm::vec2(0.8, 0.6),
                glm::vec2(-1.0, 0.0),
                glm::vec2(1.0, 1.0),
                glm::vec4(0, 229, 255, 255),
                glm::vec4(150, 150, 150, 255),
            ),
            last_instruction_execution: time::Instant::now(),
        };
    }
}

impl<'a> Scene for MainScene<'a> {
    fn game_logic(&mut self, context: &mut crate::MainContext) {
        self.robot.movement_logic(context.delta);

        if context.now.duration_since(self.last_instruction_execution).as_secs_f32() >= INSTRUCTION_PERIOD {
            self.robot.run_instruction();
            self.last_instruction_execution = context.now;
        }
    }

    fn generate_output(&mut self, context: &mut crate::MainContext) {
        let simple_shape_renderer = &mut context.simple_shape_renderer;
        self.robot.draw(simple_shape_renderer);
        self.graph.draw(&context.proj_matrix, simple_shape_renderer);
    }

    fn handle_input(&mut self, context: &mut crate::MainContext) {
        
    }
}
