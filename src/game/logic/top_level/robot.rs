use crate::game::logic::{INSTRUCTION_PERIOD, physics::RobotPhysics, top_level::{compute_context::ComputeContext, computer::{Computer, MEMORY_SIZE, X14, X15}, display_module::DisplayModule, instruction::{Instruction, Syscall}, mining_module::MiningModule, world::World}};
use nalgebra_glm as glm;

const POSITION_TOLERANCE: f32 = 0.02;
const SPEED_TOLERANCE: f32 = 0.01;

#[derive(Debug, Clone)]
pub struct RobotLogic {
    pub context: ComputeContext,
    pub time_to_next_instruction: f32,
    pub computer: Computer,
    pub mining: MiningModule,
    pub watched: bool,
}

impl RobotLogic {
    pub fn new(instructions: [Instruction; MEMORY_SIZE]) -> Self {
        Self {
            context: ComputeContext::Computer,
            time_to_next_instruction: INSTRUCTION_PERIOD,
            computer: Computer::new(instructions),
            mining: MiningModule::new(16),
            watched: false,
        }
    }
}

pub struct RobotJoin<'a> {
    position: glm::Vec2,
    speed: glm::Vec2,
    target: &'a mut glm::Vec2,
    context: &'a mut ComputeContext,
    time_to_next_instruction: &'a mut f32,
    computer: &'a mut Computer,
    display_module: &'a mut DisplayModule,
    mining: &'a mut MiningModule,
}

impl<'a> RobotJoin<'a> {
    pub fn new(robot: &'a mut RobotLogic, display_module: &'a mut DisplayModule, target: &'a mut glm::Vec2, physics: &RobotPhysics) -> Self {
        Self {
            position: physics.position,
            speed: physics.speed,
            target,
            context: &mut robot.context,
            time_to_next_instruction: &mut robot.time_to_next_instruction,
            computer: &mut robot.computer,
            display_module,
            mining: &mut robot.mining,
        }
    }

    fn move_syscall(&mut self, syscall: Syscall) {
        *self.target = self.position.map(|x| x.round()) + match syscall {
            Syscall::MoveBottom => glm::vec2(0.0, -1.0),
            Syscall::MoveLeft => glm::vec2(-1.0, 0.0),
            Syscall::MoveRight => glm::vec2(1.0, 0.0),
            Syscall::MoveTop => glm::vec2(0.0, 1.0),
            _ => glm::vec2(0.0, 0.0),
        };
    }

    fn target_achieved(&self) -> bool {
        let sq = |x: f32| x * x;

        let diff = *self.target - self.position;
        let distance2 = glm::length2(&diff);
        let speed_val2 = glm::length2(&self.speed);

        return distance2 <= sq(POSITION_TOLERANCE) && speed_val2 <= sq(SPEED_TOLERANCE);
    }

    fn movement_context(&self) -> Option<u8> {
        self.target_achieved().then_some(1)
    }

    pub fn update(&mut self, delta: f32, world: &mut World) {
        let amount_instructions_f32 = ((delta - *self.time_to_next_instruction) / INSTRUCTION_PERIOD).ceil().max(0.0);
        let amount_instructions = amount_instructions_f32 as usize;
        *self.time_to_next_instruction += amount_instructions_f32 * INSTRUCTION_PERIOD - delta;

        for _ in 0..amount_instructions {
            match self.context {
                ComputeContext::Computer => {
                    let comp = self.computer.run_computer();
                    if let Some(sys) = comp {
                        match sys {
                            Syscall::MoveBottom | Syscall::MoveLeft | Syscall::MoveRight | Syscall::MoveTop => {
                                *self.context = ComputeContext::Movement;
                                self.move_syscall(sys);
                            },
                            Syscall::Mine | Syscall::Stored | Syscall::MaxStorage | Syscall::Scan | Syscall::Release => {
                                *self.context = ComputeContext::Mining;
                                self.mining.receive_syscall(sys);
                            },
                            Syscall::Display => {
                                *self.context = ComputeContext::Display;
                                self.display_module.receive_syscall(
                                    if self.computer.read_reg(X14) != 0 {
                                        Some(self.computer.read_reg(X15))
                                    } else {
                                        None
                                    }
                                );
                            },
                            _ => {}
                        }
                    }
                },
                ComputeContext::Movement => {
                    if let Some(ret) = self.movement_context() {
                        *self.context = ComputeContext::Computer;
                        self.computer.write_reg(X15, ret);
                    }
                },
                ComputeContext::Display => {
                    if let Some(ret) = self.display_module.update() {
                        *self.context = ComputeContext::Computer;
                        self.computer.write_reg(X15, ret);
                    }
                },
                ComputeContext::Mining => {
                    if let Some(ret) = self.mining.update(world, self.position) {
                        *self.context = ComputeContext::Computer;
                        self.computer.write_reg(X15, ret);
                    }
                },
            }
        }
    }
}