
use nalgebra_glm::{self as glm};

use crate::{game::{compute_context::ComputeContext, computer::{Computer, X}, display_computer::DisplayComputer, instruction::{Instruction, Syscall}, mining_computer::MiningComputer, movement_computer::MovementComputer, world::World}, graphics::renderer::{fonts::Font, positioning::PositionMode, simple_shape_renderer::SimpleShapeRenderer}};


const ROBOT_SIZE: f32 = 0.8;
const WEEL_DIAMETER: f32 = 0.26;
const WEEL_DISTANCE: f32 = 0.37;
const ROBOT_DEPTH: u16 = 5000;
const WEELS_DEPTH: u16 = ROBOT_DEPTH + 1;
const WEEL_COLOR: [u8; 3] = [45, 40, 40];
const EPS: f32 = 0.0001;

const MAX_ACCELLERATION: f32 = 1.0;
const DRAG: f32 = 1.0;

#[derive(Debug)]
pub struct Robot {
    movement_computer: MovementComputer,
    mining_computer: MiningComputer,
    display_computer: DisplayComputer,
    computer: Computer,
    color: glm::U8Vec3,
    context: ComputeContext,
}

impl Robot {
    pub fn new(position: glm::Vec2) -> Self {
        Self {
            computer: Computer::default(),
            movement_computer: MovementComputer::new(position),
            mining_computer: MiningComputer::new(16),
            color: glm::vec3(120, 200, 190),
            context: ComputeContext::Computer,
            display_computer: DisplayComputer::new(),
        }
    }

    pub fn set_instruction(mut self, idx: usize, instruction: Instruction) -> Self {
        self.computer.set_instruction(idx, instruction);
        return self;
    }

    // pub fn draw_vec(&self, simple_shape_renderer: &mut SimpleShapeRenderer, vec: glm::Vec2, color: glm::U8Vec4) {
    //     let size = glm::length(&vec);
    //     simple_shape_renderer.send_rotated_rectangle(
    //         self.position,
    //         glm::vec2(0.5, 0.04) * size,
    //         PositionMode::CenterLeft,
    //         glm::vec2(-vec.y, vec.x),
    //         color, 2000, 0);
        
    //     simple_shape_renderer.send_symmetrical_triangle(
    //         self.position + vec * 0.6,
    //         self.position + vec * 0.5,
    //         size * 0.1,
    //         color, 2000, 0);
    // }

    // pub fn draw_compute_info(&self, font: &mut Font, simple_shape_renderer: &mut SimpleShapeRenderer) {
    //     self.instructions.send_draw(
    //         glm::vec2(1.0, 0.01),
    //         glm::vec2(0.98, 0.78), 
    //         PositionMode::BottomCenter,
    //         font, simple_shape_renderer,
    //         Some(self.memory.read_memory(PROGRAM_COUNTER_IDX) as usize),
    //         false,
    //     );

    //     self.memory.send_draw(
    //         glm::vec2(1.0, -0.05),
    //         glm::vec2(0.9, 0.7),
    //         PositionMode::TopCenter,
    //         font, simple_shape_renderer,
    //         4,
    //     );

    //     let code_background = glm::vec4(10, 10, 20, 255);
    //     simple_shape_renderer.send_rectangle(
    //         glm::vec2(1.0, 0.0),
    //         glm::vec2(1.0, 1.6),
    //         PositionMode::Center,
    //         code_background,
    //         50001, 1,
    //     );
    // }


    pub fn draw(&self, simple_shape_renderer: &mut SimpleShapeRenderer, font: &mut Font) {
        // self.draw_vec(simple_shape_renderer, self.acc, glm::make_vec4(&COLOR_DATA[0]));
        // self.draw_vec(simple_shape_renderer, self.drag, glm::make_vec4(&COLOR_DATA[1]));

        let weel_offsets = [
            glm::vec2(-1.0, -1.0),
            glm::vec2( 1.0, -1.0),
            glm::vec2( 1.0,  1.0),
            glm::vec2(-1.0,  1.0),
        ];

        for weel_offset in weel_offsets {
            simple_shape_renderer.send_ellipse(
                self.movement_computer.get_position()  + weel_offset * WEEL_DISTANCE,
                glm::vec2(WEEL_DIAMETER, WEEL_DIAMETER),
                PositionMode::Center,
                glm::vec4(WEEL_COLOR[0], WEEL_COLOR[1], WEEL_COLOR[2], 255),
                WEELS_DEPTH, 0
            );
        }

        simple_shape_renderer.send_rectangle(
            self.movement_computer.get_position(),
            glm::vec2(ROBOT_SIZE, ROBOT_SIZE),
            PositionMode::Center,
            glm::vec4(self.color.x, self.color.y, self.color.z, 255),
            ROBOT_DEPTH, 0
        );

        self.display_computer.send(font, self.movement_computer.get_position());
    }

    pub fn update(&mut self, delta: f32, is_instruction_tick: bool, world : &mut World) {
        // println!("{}", self.computer.read_reg(X[1]));
        match self.context {
            ComputeContext::Computer => {
                if is_instruction_tick {
                    let comp = self.computer.run_computer();
                    if let Some(sys) = comp {
                        match sys {
                            Syscall::MoveBottom | Syscall::MoveLeft | Syscall::MoveRight | Syscall::MoveTop => {
                                self.context = ComputeContext::MovementComputer;
                                self.movement_computer.receive_syscall(sys);
                            },
                            Syscall::Mine | Syscall::Stored | Syscall::MaxStorage => {
                                self.context = ComputeContext::MiningComputer;
                                self.mining_computer.receive_syscall(sys);
                            },
                            Syscall::Display => {
                                self.context = ComputeContext::DisplayComputer;
                                self.display_computer.receive_syscall(
                                    if self.computer.read_reg(X[14]) != 0 {
                                        Some(self.computer.read_reg(X[15]))
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
                if let Some(ret) = self.movement_computer.update() {
                    self.context = ComputeContext::Computer;
                    self.computer.write_reg(X[15], ret);
                }
            },
            ComputeContext::MiningComputer => {
                if let Some(ret) = self.mining_computer.update(
                    self.movement_computer.get_position(),
                    world, delta, is_instruction_tick,
                ) {
                    self.context = ComputeContext::Computer;
                    self.computer.write_reg(X[15], ret);
                }
            },
            ComputeContext::DisplayComputer => {
                if let Some(ret) = self.display_computer.update(is_instruction_tick) {
                    self.context = ComputeContext::Computer;
                    self.computer.write_reg(X[15], ret);
                }
            }
        };

        self.movement_logic(delta, self.movement_computer.calculate_accelleration());
    }

    fn movement_logic(&mut self, delta: f32, acc: glm::Vec2) {
        let sq = |x| x * x;

        let acc_mag = glm::length(&acc);
        let acc_adjust = acc_mag.min(MAX_ACCELLERATION);
        let acc = acc / (acc_mag + EPS) * acc_adjust;

        let position = self.movement_computer.get_position();
        let speed = self.movement_computer.get_speed();

        let speed_mag = glm::length(&speed);
        let norm_speed = speed / (speed_mag + EPS);
        let drag_vector = -norm_speed * sq(speed_mag) * DRAG;

        let all_acc = acc + drag_vector;

        let next_position = position + speed * delta + all_acc * sq(delta) * 0.5;
        let next_speed = speed + all_acc * delta;

        self.movement_computer.update_sensor(next_position, next_speed);
    }
}