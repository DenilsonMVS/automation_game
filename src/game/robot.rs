
use nalgebra_glm::{self as glm};

use crate::{game::instruction::Instruction, graphics::renderer::{positioning::PositionMode, simple_shape_renderer::SimpleShapeRenderer}};

const MEMORY_RANK: usize = 4;
const MEMORY_SIZE: usize = 1 << MEMORY_RANK;
const MEMORY_MASK: usize = MEMORY_SIZE - 1;
const PROGRAM_COUNTER_IDX: usize = 0;
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
pub struct Memory<const SIZE: usize> {
    memory: [u8; SIZE],
}

impl<const SIZE: usize> Memory<SIZE> {
    fn new() -> Self {
        Self {
            memory: [0; SIZE]
        }
    }

    fn read_memory<T: Into<usize>>(&self, address: T) -> u8 {
        self.memory[address.into() & (SIZE - 1)]
    }

    fn write_memory<T: Into<usize>>(&mut self, address: T, value: u8) {
        self.memory[address.into() & (SIZE - 1)] = value;
    }

    pub fn inst1<T, F>(&mut self, input_addr: T, out_addr: T, func: F)
    where
        T: Into<usize> + Copy,
        F: Fn(u8) -> u8, 
    {
        let val = self.read_memory(input_addr);
        let result = func(val);
        self.write_memory(out_addr, result);
    }

    pub fn inst2<T, F>(&mut self, input_addr1: T, input_addr2: T, out_addr: T, func: F)
    where
        T: Into<usize> + Copy,
        F: Fn(u8, u8) -> u8, 
    {
        let val1 = self.read_memory(input_addr1);
        let val2 = self.read_memory(input_addr2);
        let result = func(val1, val2);
        self.write_memory(out_addr, result);
    }

    pub fn cond_jump<T, F>(&mut self, input_addr1: T, input_addr2: T, jump: u8, func: F)
    where
        T: Into<usize> + Copy,
        F: Fn(u8, u8) -> bool,
    {
        let val1 = self.read_memory(input_addr1);
        let val2 = self.read_memory(input_addr2);
        if func(val1, val2) {
            self.write_memory(PROGRAM_COUNTER_IDX, jump.wrapping_sub(1));
        }
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
    memory: Memory<MEMORY_SIZE>,
    instructions: [Instruction; MEMORY_SIZE],
    color: glm::U8Vec3,
}

impl Robot {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
            position: glm::vec2(0.0, 0.0),
            speed: glm::vec2(0.0, 0.0),
            target: glm::vec2(0.0, 0.0),
            instructions: [Instruction::NoOp; MEMORY_SIZE],
            color: glm::vec3(200, 200, 190),
        }
    }

    pub fn set_instruction(mut self, idx: usize, instruction: Instruction) -> Self {
        self.instructions[(idx as usize) & MEMORY_MASK] = instruction;
        return self;
    }

    pub fn read_instruction(&self, idx: u8) -> Instruction {
        self.instructions[(idx as usize) & MEMORY_MASK]
    }

    pub fn run_instruction(&mut self) {
        if self.on_movement_instruction() {
            return;
        }

        let center = self.position.map(|x| x.round());
        match self.read_instruction(self.memory.read_memory(PROGRAM_COUNTER_IDX)) {
            Instruction::MoveBottom => self.target = center + glm::vec2( 0.0, -1.0),
            Instruction::MoveTop    => self.target = center + glm::vec2( 0.0,  1.0),
            Instruction::MoveLeft   => self.target = center + glm::vec2(-1.0,  0.0),
            Instruction::MoveRight  => self.target = center + glm::vec2( 1.0,  0.0),

            Instruction::Write(value, addr) => self.memory.write_memory(addr, value),
            Instruction::Goto(inst) => self.memory.write_memory(PROGRAM_COUNTER_IDX, inst.wrapping_sub(1)),
            Instruction::Move(src, dest) => self.memory.inst1(src, dest, |x| x),
           
            Instruction::Add(addr1, addr2, dest) => self.memory.inst2(addr1, addr2, dest, |a, b| a.wrapping_add(b)),
            Instruction::Sub(addr1, addr2, dest) => self.memory.inst2(addr1, addr2, dest, |a, b| a.wrapping_sub(b)),
            Instruction::Mul(addr1, addr2, dest) => self.memory.inst2(addr1, addr2, dest, |a, b| a.wrapping_mul(b)),
            Instruction::Div(addr1, addr2, dest) => self.memory.inst2(addr1, addr2, dest, |a, b| a / replace_zero(b,1)),
            Instruction::Rem(addr1, addr2, dest) => self.memory.inst2(addr1, addr2, dest, |a, b| a % replace_zero(b,1)),
           
            Instruction::And(addr1, addr2, dest) => self.memory.inst2(addr1, addr2, dest, |a, b| (a > 0 && b > 0) as u8),
            Instruction::Or(addr1, addr2, dest) => self.memory.inst2(addr1, addr2, dest, |a, b| (a > 0 || b > 0) as u8),
            Instruction::Diff(addr1, addr2, dest) => self.memory.inst2(addr1, addr2, dest, |a, b| (a != b) as u8),
            Instruction::Eq(addr1, addr2, dest) => self.memory.inst2(addr1, addr2, dest, |a, b| (a == b) as u8),
            Instruction::Not(addr, dest) => self.memory.inst1(addr, dest, |x| (x == 0) as u8),
            Instruction::Greater(addr1, addr2, dest) => self.memory.inst2(addr1, addr2, dest, |a, b| (a > b) as u8),
            Instruction::GreaterEq(addr1, addr2, dest) => self.memory.inst2(addr1, addr2, dest, |a, b| (a >= b) as u8),
            Instruction::Smaller(addr1, addr2, dest) => self.memory.inst2(addr1, addr2, dest, |a, b| (a < b) as u8),
            Instruction::SmallerEq(addr1, addr2, dest) => self.memory.inst2(addr1, addr2, dest, |a, b| (a <= b) as u8),

            Instruction::BitAnd(addr1, addr2, dest) => self.memory.inst2(addr1, addr2, dest, |a, b| a & b),
            Instruction::BitOr(addr1, addr2, dest) => self.memory.inst2(addr1, addr2, dest, |a, b| a | b),
            Instruction::BitXor(addr1, addr2, dest) => self.memory.inst2(addr1, addr2, dest, |a, b| a ^ b),
            Instruction::BitEq(addr1, addr2, dest) => self.memory.inst2(addr1, addr2, dest, |a, b| !(a ^ b)),
            Instruction::BitNot(addr, dest) => self.memory.inst1(addr, dest, |x| !x),

            Instruction::EqJump(addr1, addr2, dest) => self.memory.cond_jump(addr1, addr2, dest, |a, b| a == b),
            Instruction::DiffJump(addr1, addr2, dest) => self.memory.cond_jump(addr1, addr2, dest, |a, b| a != b),
            Instruction::GreaterJump(addr1, addr2, dest) => self.memory.cond_jump(addr1, addr2, dest, |a, b| a > b),
            Instruction::GreaterEqJump(addr1, addr2, dest) => self.memory.cond_jump(addr1, addr2, dest, |a, b| a >= b),
            Instruction::SmallerJump(addr1, addr2, dest) => self.memory.cond_jump(addr1, addr2, dest, |a, b| a < b),
            Instruction::SmallerEqJump(addr1, addr2, dest) => self.memory.cond_jump(addr1, addr2, dest, |a, b| a <= b),
            _ => {}
        }

        self.memory.write_memory(PROGRAM_COUNTER_IDX, self.memory.read_memory(PROGRAM_COUNTER_IDX).wrapping_add(1));
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