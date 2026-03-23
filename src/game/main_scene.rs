use std::time;

use crate::game::{instruction::{Instruction, RegisterAddress}, robot::Robot, scene::Scene};

const INSTRUCTION_PERIOD: f32 = 0.25;

#[derive(Debug)]
pub struct MainScene {
    robot: Robot,
    last_instruction_execution: time::Instant,
}

impl MainScene {
    pub fn new() -> Self {
        let x0 = RegisterAddress::new(0);
        let x2 = RegisterAddress::new(2);
        return Self { 
            robot: Robot::new()
                .set_instruction(0, Instruction::new_lui(x2, 2))
                .set_instruction(1, Instruction::new_sys(3))
                .set_instruction(2, Instruction::new_subi(x2, x2, 1))
                .set_instruction(3, Instruction::new_bne(x2, x0, 14))

                .set_instruction(4, Instruction::new_sys(1))

                .set_instruction(5, Instruction::new_lui(x2, 2))
                .set_instruction(6, Instruction::new_sys(4))
                .set_instruction(7, Instruction::new_subi(x2, x2, 1))
                .set_instruction(8, Instruction::new_bne(x2, x0, 14))

                .set_instruction(9, Instruction::new_sys(2)),
            last_instruction_execution: time::Instant::now(),
        };
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
