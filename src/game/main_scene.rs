use std::time;

use crate::game::{instruction::Instruction, robot::Robot, scene::Scene};

const INSTRUCTION_PERIOD: f32 = 0.25;

#[derive(Debug)]
pub struct MainScene {
    robot: Robot,
    last_instruction_execution: time::Instant,
}

impl MainScene {
    pub fn new() -> Self {
        Self { 
            robot: Robot::new()
                .set_instruction(0, Instruction::Write(10, 10))
                .set_instruction(1, Instruction::Write(1, 11))
                .set_instruction(2, Instruction::Write(0, 12))

                .set_instruction(3, Instruction::MoveLeft)
                .set_instruction(4, Instruction::Sub(10, 11, 10))
                .set_instruction(5, Instruction::DiffJump(10, 12, 3))

                .set_instruction(6, Instruction::MoveTop)

                .set_instruction(7, Instruction::Write(10, 10))
                .set_instruction(8, Instruction::MoveRight)
                .set_instruction(9, Instruction::Sub(10, 11, 10))
                .set_instruction(10, Instruction::DiffJump(10, 12, 8))

                .set_instruction(11, Instruction::MoveBottom),
            last_instruction_execution: time::Instant::now(),
        }
    }
}

impl Scene for MainScene {
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
    }

    fn handle_input(&mut self, context: &mut crate::MainContext) {
        
    }
}
