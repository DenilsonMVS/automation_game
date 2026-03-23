
use nalgebra_glm::{self as glm};

use crate::{game::instruction::{Instruction, UnpackedInstruction}, graphics::renderer::{positioning::PositionMode, simple_shape_renderer::SimpleShapeRenderer}};

const MEMORY_RANK: usize = 4;
const MEMORY_SIZE: usize = 1 << MEMORY_RANK;
const ZERO_IDX: usize = 0;
pub const PROGRAM_COUNTER_IDX: usize = 1;
const ROBOT_SIZE: f32 = 0.9;
const WEEL_DIAMETER: f32 = 0.3;
const WEEL_DISTANCE: f32 = 0.43;
const ROBOT_DEPTH: u16 = 5000;
const WEELS_DEPTH: u16 = ROBOT_DEPTH + 1;
const WEEL_COLOR: [u8; 3] = [45, 40, 40];
const EPS: f32 = 0.0001;
const POSITION_TOLERANCE: f32 = 0.02;
const SPEED_TOLERANCE: f32 = 0.01;

const MAX_ACCELLERATION: f32 = 1.0;
const DRAG: f32 = 1.0;

const PROPORTIONAL: f32 = 10.0;
const DERIVATIVE: f32 = -6.0;

#[derive(Debug)]
pub struct Memory<T: Copy + Default, const SIZE: usize> {
    memory: [T; SIZE],
}

impl<T: Copy + Default, const SIZE: usize> Memory<T, SIZE> {
    fn new() -> Self {
        Self {
            memory: [T::default(); SIZE]
        }
    }

    fn read_memory<ADDR: Into<usize>>(&self, address: ADDR) -> T {
        self.memory[address.into() & (SIZE - 1)]
    }

    fn write_memory<ADDR: Into<usize>>(&mut self, address: ADDR, value: T) {
        let address = address.into();
        if address != ZERO_IDX {
            self.memory[address & (SIZE - 1)] = value;
        }
    }

    fn write_instruction_memory<ADDR: Into<usize>>(&mut self, address: ADDR, value: T) {
        self.memory[address.into() & (SIZE - 1)] = value;
    }

    pub fn inst1<ADDR, F>(&mut self, input_addr: ADDR, out_addr: ADDR, func: F)
    where
        ADDR: Into<usize> + Copy,
        F: Fn(T) -> T, 
    {
        let val = self.read_memory(input_addr);
        let result = func(val);
        self.write_memory(out_addr, result);
    }

    pub fn inst2<ADDR, F>(&mut self, input_addr1: ADDR, input_addr2: ADDR, out_addr: ADDR, func: F)
    where
        ADDR: Into<usize> + Copy,
        F: Fn(T, T) -> T, 
    {
        let val1 = self.read_memory(input_addr1);
        let val2 = self.read_memory(input_addr2);
        let result = func(val1, val2);
        self.write_memory(out_addr, result);
    }

    pub fn inst3<ADDR, F>(&mut self, input_addr1: ADDR, input_addr2: ADDR, input_addr3: ADDR, out_addr: ADDR, func: F)
    where
        ADDR: Into<usize> + Copy,
        F: Fn(T, T, T) -> T, 
    {
        let val1 = self.read_memory(input_addr1);
        let val2 = self.read_memory(input_addr2);
        let val3 = self.read_memory(input_addr3);
        let result = func(val1, val2, val3);
        self.write_memory(out_addr, result);
    }
}

fn replace_zero(value: u8, replacement: u8) -> u8 {
    if value == 0 {
        replacement
    } else {
        value
    }
}

#[derive(Debug)]
pub struct Robot {
    position: glm::Vec2,
    speed: glm::Vec2,
    target: glm::Vec2,
    memory: Memory<u8, MEMORY_SIZE>,
    instructions: Memory<Instruction, MEMORY_SIZE>,
    color: glm::U8Vec3,
}

impl Robot {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
            position: glm::vec2(0.0, 0.0),
            speed: glm::vec2(0.0, 0.0),
            target: glm::vec2(0.0, 0.0),
            instructions: Memory::new(),
            color: glm::vec3(200, 200, 190),
        }
    }

    pub fn set_instruction(mut self, idx: usize, instruction: Instruction) -> Self {
        self.instructions.write_instruction_memory(idx, instruction);
        return self;
    }

    pub fn read_instruction(&self, idx: u8) -> Instruction {
        self.instructions.read_memory(idx)
    }

    pub fn run_instruction(&mut self) {
        if self.on_movement_instruction() {
            return;
        }

        let unpacked_instruction = self.read_instruction(self.memory.read_memory(PROGRAM_COUNTER_IDX)).unpack();
        match unpacked_instruction {
            UnpackedInstruction::Lui(rd, imm) =>
                self.memory.write_memory(rd.get_raw(), imm),
            UnpackedInstruction::AddI(rs, rd, imm) =>
                self.memory.inst1(rs.get_raw(), rd.get_raw(), |x| x.wrapping_add(imm)),
            UnpackedInstruction::SubI(rs, rd, imm) =>
                self.memory.inst1(rs.get_raw(), rd.get_raw(), |x| x.wrapping_sub(imm)),
            UnpackedInstruction::Add(r1, r2, rd) =>
                self.memory.inst2(r1.get_raw(), r2.get_raw(), rd.get_raw(), |a, b| a.wrapping_add(b)),
            UnpackedInstruction::Sub(r1, r2, rd) =>
                self.memory.inst2(r1.get_raw(), r2.get_raw(), rd.get_raw(), |a, b| a.wrapping_sub(b)),
            UnpackedInstruction::Mul(r1, r2, rd) =>
                self.memory.inst2(r1.get_raw(), r2.get_raw(), rd.get_raw(), |a, b| a.wrapping_mul(b)),
            UnpackedInstruction::Div(r1, r2, rd) =>
                self.memory.inst2(r1.get_raw(), r2.get_raw(), rd.get_raw(), |a, b| a / replace_zero(b, 1)),
            UnpackedInstruction::Rem(r1, r2, rd) =>
                self.memory.inst2(r1.get_raw(), r2.get_raw(), rd.get_raw(), |a, b| a % replace_zero(b, 1)),
            UnpackedInstruction::Slt(r1, r2, rd) =>
                self.memory.inst2(r1.get_raw(), r2.get_raw(), rd.get_raw(), |a, b| (a < b) as u8),
            UnpackedInstruction::Sle(r1, r2, rd) =>
                self.memory.inst2(r1.get_raw(), r2.get_raw(), rd.get_raw(), |a, b| (a <= b) as u8),
            UnpackedInstruction::Beq(r1, r2, imm) =>
                self.memory.inst3(r1.get_raw(), r2.get_raw(), PROGRAM_COUNTER_IDX as u16, PROGRAM_COUNTER_IDX as u16, |a, b, c| if a == b { c.wrapping_add(imm).wrapping_sub(1) } else { c }),
            UnpackedInstruction::Bne(r1, r2, imm) =>
                self.memory.inst3(r1.get_raw(), r2.get_raw(), PROGRAM_COUNTER_IDX as u16, PROGRAM_COUNTER_IDX as u16, |a, b, c| if a != b { c.wrapping_add(imm).wrapping_sub(1) } else { c }),
            UnpackedInstruction::Blt(r1, r2, imm) =>
                self.memory.inst3(r1.get_raw(), r2.get_raw(), PROGRAM_COUNTER_IDX as u16, PROGRAM_COUNTER_IDX as u16, |a, b, c| if a < b { c.wrapping_add(imm).wrapping_sub(1) } else { c }),
            UnpackedInstruction::Ble(r1, r2, imm) =>
                self.memory.inst3(r1.get_raw(), r2.get_raw(), PROGRAM_COUNTER_IDX as u16, PROGRAM_COUNTER_IDX as u16, |a, b, c| if a <= b { c.wrapping_add(imm).wrapping_sub(1) } else { c }),
            UnpackedInstruction::Jr(rs) =>
                self.memory.inst1(rs.get_raw(), PROGRAM_COUNTER_IDX as u16, |x| x.wrapping_sub(1)),

            UnpackedInstruction::Sys(code) => {
                let center = self.position.map(|x| x.round());
                match code {
                    1 => self.target = center + glm::vec2( 0.0,  1.0),
                    2 => self.target = center + glm::vec2( 0.0, -1.0),
                    3 => self.target = center + glm::vec2(-1.0,  0.0),
                    4 => self.target = center + glm::vec2( 1.0,  0.0),
                    _ => {}
                };
            }
        }

        self.memory.inst1(PROGRAM_COUNTER_IDX, PROGRAM_COUNTER_IDX, |x| x.wrapping_add(1));
    }

    fn on_movement_instruction(&self) -> bool {
        let sq = |x: f32| x * x;

        let diff = self.target - self.position;
        let distance2 = glm::length2(&diff);
        let speed_val2 = glm::length2(&self.speed);

        return distance2 > sq(POSITION_TOLERANCE) || speed_val2 > sq(SPEED_TOLERANCE);
    }

    pub fn draw(&self, simple_shape_renderer: &mut SimpleShapeRenderer) {
        let weel_offsets = [
            glm::vec2(-1.0, -1.0),
            glm::vec2( 1.0, -1.0),
            glm::vec2( 1.0,  1.0),
            glm::vec2(-1.0,  1.0),
        ];

        for weel_offset in weel_offsets {
            simple_shape_renderer.send_ellipse(
                self.position + weel_offset * WEEL_DISTANCE,
                glm::vec2(WEEL_DIAMETER, WEEL_DIAMETER),
                PositionMode::Center,
                glm::vec4(WEEL_COLOR[0], WEEL_COLOR[1], WEEL_COLOR[2], 255),
                WEELS_DEPTH,
            );
        }

        simple_shape_renderer.send_rectangle(
            self.position,
            glm::vec2(ROBOT_SIZE, ROBOT_SIZE),
            PositionMode::Center,
            glm::vec4(self.color.x, self.color.y, self.color.z, 255),
            ROBOT_DEPTH,
        );
    }

    pub fn movement_logic(&mut self, delta: f32) {
        let sq = |x| x * x;

        if self.on_movement_instruction() {
            let diff = self.target - self.position;
            let p_term = diff * PROPORTIONAL;
            let d_term = self.speed * DERIVATIVE;
            let target_accel_vec = p_term + d_term;

            let accel_mag = glm::length(&target_accel_vec);

            let accel_adjust = accel_mag.min(MAX_ACCELLERATION);
            let target_accel_vec_applied = target_accel_vec / accel_mag * accel_adjust;

            let speed_val = glm::length(&self.speed);
            let norm_speed = self.speed / (speed_val + EPS);
            let drag_vector = -norm_speed * sq(speed_val) * DRAG;
            let all_acceleration = drag_vector + target_accel_vec_applied;

            self.position += self.speed * delta + all_acceleration * sq(delta) * 0.5;
            self.speed += all_acceleration * delta;

        } else if self.speed == glm::vec2(0.0, 0.0) {
            return;
        
        } else {
            let speed_val = glm::length(&self.speed);
            let norm_speed = self.speed / (speed_val + EPS);
            let acc_vec = -norm_speed * MAX_ACCELLERATION;

            let time_to_stop = speed_val / MAX_ACCELLERATION;
            let virtual_delta = delta.min(time_to_stop);

            self.position += self.speed * virtual_delta - acc_vec * sq(virtual_delta) * 0.5;
            self.speed = glm::vec2(0.0, 0.0);
        }
    }
}