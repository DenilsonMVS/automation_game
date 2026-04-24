
use nalgebra_glm::{self as glm};

use crate::{MainContext, game::logic::{compute_context::ComputeContext, computer::{Computer, MEMORY_SIZE, X14, X15}, display_module::DisplayModule, instruction::{Instruction, Syscall}, mining_module::MiningModule, movement_module::MovementModule, world::World}, ui::button_area::ButtonArea};

const EPS: f32 = 0.0001;

const MAX_ACCELLERATION: f32 = 1.0;
const DRAG: f32 = 1.0;

#[derive(Debug)]
pub struct Robot {
    movement_module: MovementModule,
    mining_module: MiningModule,
    display_module: DisplayModule,
    computer: Computer,
    color: glm::U8Vec3,
    context: ComputeContext,
}

impl Robot {
    pub fn new(position: glm::Vec2, instructions: [Instruction; MEMORY_SIZE]) -> Self {
        Self {
            computer: Computer::new(instructions),
            movement_module: MovementModule::new(position),
            mining_module: MiningModule::new(16),
            color: glm::vec3(120, 200, 190),
            context: ComputeContext::Computer,
            display_module: DisplayModule::new(),
        }
    }

    pub fn update(
        &mut self,
        is_instruction_tick: bool,
        world : &mut World,
        context: &mut MainContext,
        robot_idx: usize,
    ) {
        let delta = context.ui.get_delta();

        match self.context {
            ComputeContext::Computer => {
                if is_instruction_tick {
                    let comp = self.computer.run_computer();
                    if let Some(sys) = comp {
                        match sys {
                            Syscall::MoveBottom | Syscall::MoveLeft | Syscall::MoveRight | Syscall::MoveTop => {
                                self.context = ComputeContext::MovementComputer;
                                self.movement_module.receive_syscall(sys);
                            },
                            Syscall::Mine | Syscall::Stored | Syscall::MaxStorage | Syscall::Scan => {
                                self.context = ComputeContext::MiningComputer;
                                self.mining_module.receive_syscall(sys);
                            },
                            Syscall::Display => {
                                self.context = ComputeContext::DisplayComputer;
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
                }
            },
            ComputeContext::MovementComputer => {
                if let Some(ret) = self.movement_module.update() {
                    self.context = ComputeContext::Computer;
                    self.computer.write_reg(X15, ret);
                }
            },
            ComputeContext::MiningComputer => {
                if let Some(ret) = self.mining_module.update(
                    self.movement_module.get_position(),
                    world, delta, is_instruction_tick,
                ) {
                    self.context = ComputeContext::Computer;
                    self.computer.write_reg(X15, ret);
                }
            },
            ComputeContext::DisplayComputer => {
                if let Some(ret) = self.display_module.update(is_instruction_tick) {
                    self.context = ComputeContext::Computer;
                    self.computer.write_reg(X15, ret);
                }
            }
        };

        self.movement_logic(delta, self.movement_module.calculate_accelleration());

        let render_box = self.get_render_box();
        context.ui.push_button_area(ButtonArea::new_from_world_pos(
            render_box, move |context, _|
                context.scene.logic.robot_info_opened = Some(robot_idx),
            context.ui.get_camera(),));
    }

    pub fn get_computer(&self) -> &Computer {
        &self.computer
    }

    fn movement_logic(&mut self, delta: f32, acc: glm::Vec2) {
        let sq = |x| x * x;

        let acc_mag = glm::length(&acc);
        let acc_adjust = acc_mag.min(MAX_ACCELLERATION);
        let acc = acc / (acc_mag + EPS) * acc_adjust;

        let position = self.movement_module.get_position();
        let speed = self.movement_module.get_speed();

        let speed_mag = glm::length(&speed);
        let norm_speed = speed / (speed_mag + EPS);
        let drag_vector = -norm_speed * sq(speed_mag) * DRAG;

        let all_acc = acc + drag_vector;

        let next_position = position + speed * delta + all_acc * sq(delta) * 0.5;
        let next_speed = speed + all_acc * delta;

        self.movement_module.update_sensor(next_position, next_speed);
    }

    pub fn get_position(&self) -> glm::Vec2 {
        self.movement_module.get_position()
    }

    pub fn get_color(&self) -> glm::U8Vec4 {
        glm::vec4(self.color.x, self.color.y, self.color.z, 255)
    }

    pub fn get_display_module(&self) -> &DisplayModule {
        &self.display_module
    }
}
