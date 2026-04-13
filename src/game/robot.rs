
use nalgebra_glm::{self as glm};

use crate::{game::{graph::Graph, instruction::{Instruction, UnpackedInstruction}}, graphics::renderer::{fonts::{Font, TextRenderConfig}, positioning::PositionMode, simple_shape_renderer::SimpleShapeRenderer}};

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

impl<const SIZE: usize> Memory<Instruction, SIZE> {
    fn draw_line(
        font: &mut Font,
        position: glm::Vec2,
        instruction: Instruction,
        line_height: f32,
        glyph_space: f32,
    ) {
        let code_colors = [
            glm::vec4(0xff, 0xff, 0xff, 0xff),
            glm::vec4( 120,  255,  120, 0xff),
            glm::vec4( 120,  255,  255,  255),
            glm::vec4( 255,  120,  120, 0xff),
        ];

        let visualization = instruction.unpack().get_visualization();
        font.add_text(&TextRenderConfig::new(
            &visualization[0],
            position + glm::vec2(0.0, 0.0),
            line_height, code_colors[0],
        ).set_mode(PositionMode::TopLeft).set_group(1));
        font.add_text(&TextRenderConfig::new(
            &visualization[1],
            position + glm::vec2(glyph_space * 5.0, 0.0),
            line_height, code_colors[1],
        ).set_mode(PositionMode::TopLeft).set_group(1));
        font.add_text(&TextRenderConfig::new(
            &visualization[2],
            position + glm::vec2(glyph_space * 10.0, 0.0),
            line_height, code_colors[2],
        ).set_mode(PositionMode::TopLeft).set_group(1));
        font.add_text(&TextRenderConfig::new(
            &visualization[3],
            position + glm::vec2(glyph_space * 15.0, 0.0),
            line_height, code_colors[3],
        ).set_mode(PositionMode::TopLeft).set_group(1));   
    }

    fn draw_line_simplified(
        font: &mut Font,
        position: glm::Vec2,
        instruction: Instruction,
        line_height: f32,
    ) {
        let code_colors = glm::vec4(0xff, 0xff, 0xff, 0xff);
        let simplified_visualization = instruction.unpack().get_simplified_visualization();
        font.add_text(&TextRenderConfig::new(
            &simplified_visualization,
            position + glm::vec2(0.0, 0.0),
            line_height, code_colors,
        ).set_mode(PositionMode::TopLeft).set_group(1));
    }

    pub fn send_draw(&self,
        position: glm::Vec2,
        size: glm::Vec2,
        position_mode: PositionMode,
        font: &mut Font,
        simple_shape_renderer: &mut SimpleShapeRenderer,
        current_instruction: Option<usize>,
        simplified: bool,
    ) {
        const INTERNAL_PADDING: f32 = 0.005;
        let code_background = glm::vec4(20, 20, 40, 255);
        let highlighted_instruction = code_background + glm::vec4(50, 50, 50, 0);

        let bound_box = simple_shape_renderer.send_rectangle(
            position,
            size,
            position_mode,
            code_background,
            50000, 1,
        );

        let line_height = (size.y - INTERNAL_PADDING * 2.0) / SIZE as f32;
        let glyph_space = line_height * 0.5;
        let top_left_corner = bound_box.get_top_left();
        let first_position = top_left_corner + glm::vec2(INTERNAL_PADDING, -INTERNAL_PADDING);
        for (i, instruction) in self.memory.iter().enumerate() {
            let position = first_position - glm::vec2(0.0, line_height * i as f32);
            if simplified {
                Self::draw_line_simplified(font, position, *instruction, line_height);
            } else {
                Self::draw_line(font, position, *instruction, line_height, glyph_space);
            }
        }

        if let Some(i) = current_instruction {
            let position = first_position - glm::vec2(0.0, line_height * (i & (SIZE - 1)) as f32);
            simple_shape_renderer.send_rectangle(
                position,
                glm::vec2(size.x - INTERNAL_PADDING * 2.0, line_height),
                PositionMode::TopLeft,
                highlighted_instruction,
                49999, 1,
            );
        }
    }
}

impl<const SIZE: usize> Memory<u8, SIZE> {
    pub fn send_draw(&self,
        position: glm::Vec2,
        size: glm::Vec2,
        position_mode: PositionMode,
        font: &mut Font,
        simple_shape_renderer: &mut SimpleShapeRenderer,
        units_per_line: usize,
    ) {
        const INTERNAL_PADDING: f32 = 0.005;
        let code_background = glm::vec4(20, 20, 40, 255);
        let quad_background = code_background + glm::vec4(30, 30, 50, 0);
        let value_color = glm::vec4( 250,  250,  120, 0xff);

        let bound_box = simple_shape_renderer.send_rectangle(
            position,
            size,
            position_mode,
            code_background,
            50000, 1,
        );

        let lines = SIZE / units_per_line;
        let columns = SIZE / lines;
        let unit_size = glm::vec2(
            (size.x - INTERNAL_PADDING * (lines + 1) as f32) / (lines as f32),
            (size.y - INTERNAL_PADDING * (columns + 1) as f32) / (columns as f32),
        );

        let top_left_corner = bound_box.get_top_left();
        let first_position = top_left_corner + glm::vec2(INTERNAL_PADDING, -INTERNAL_PADDING);

        for (i, value) in self.memory.iter().enumerate() {
            let line = i / columns;
            let column = i % columns;
            let position = first_position +
                glm::vec2(
                    column as f32 * (INTERNAL_PADDING + unit_size.x),
                    -(line as f32 * (INTERNAL_PADDING + unit_size.y)),
                );

            simple_shape_renderer.send_rectangle(
                position,
                unit_size,
                PositionMode::TopLeft,
                quad_background,
                40000, 1,
            );

            let text_center = position + glm::vec2(unit_size.x, -unit_size.y) * 0.5;
            let formatted = format!("{:02x}", value);
            font.add_text(&TextRenderConfig::new(
                &formatted,
                text_center,
                unit_size.y,
                value_color,
            ).set_group(1));
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
    memory: Memory<u8, MEMORY_SIZE>,
    instructions: Memory<Instruction, MEMORY_SIZE>,
    color: glm::U8Vec3,
    acc: glm::Vec2,
    drag: glm::Vec2,
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
            acc: glm::vec2(0.0, 0.0),
            drag: glm::vec2(0.0, 0.0),
        }
    }

    pub fn set_instruction(mut self, idx: usize, instruction: Instruction) -> Self {
        self.instructions.write_instruction_memory(idx, instruction);
        return self;
    }

    pub fn run_instruction(&mut self) {
        if self.on_movement_instruction() {
            return;
        }

        let unpacked_instruction = self.instructions.read_memory(self.memory.read_memory(PROGRAM_COUNTER_IDX)).unpack();
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
            UnpackedInstruction::Seq(r1, r2, rd) =>
                self.memory.inst2(r1.get_raw(), r2.get_raw(), rd.get_raw(), |a, b| (a == b) as u8),
            UnpackedInstruction::Slt(r1, r2, rd) =>
                self.memory.inst2(r1.get_raw(), r2.get_raw(), rd.get_raw(), |a, b| (a < b) as u8),
            UnpackedInstruction::Sle(r1, r2, rd) =>
                self.memory.inst2(r1.get_raw(), r2.get_raw(), rd.get_raw(), |a, b| (a <= b) as u8),
            UnpackedInstruction::Beq(r1, r2, imm) =>
                self.memory.inst3(r1.get_raw(), r2.get_raw(), PROGRAM_COUNTER_IDX as u16, PROGRAM_COUNTER_IDX as u16, |a, b, c| if a == b { c.wrapping_add(imm) } else { c }),
            UnpackedInstruction::Bne(r1, r2, imm) =>
                self.memory.inst3(r1.get_raw(), r2.get_raw(), PROGRAM_COUNTER_IDX as u16, PROGRAM_COUNTER_IDX as u16, |a, b, c| if a != b { c.wrapping_add(imm) } else { c }),
            UnpackedInstruction::Blt(r1, r2, imm) =>
                self.memory.inst3(r1.get_raw(), r2.get_raw(), PROGRAM_COUNTER_IDX as u16, PROGRAM_COUNTER_IDX as u16, |a, b, c| if a < b { c.wrapping_add(imm) } else { c }),
            UnpackedInstruction::Ble(r1, r2, imm) =>
                self.memory.inst3(r1.get_raw(), r2.get_raw(), PROGRAM_COUNTER_IDX as u16, PROGRAM_COUNTER_IDX as u16, |a, b, c| if a <= b { c.wrapping_add(imm) } else { c }),

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

    pub fn draw_vec(&self, simple_shape_renderer: &mut SimpleShapeRenderer, vec: glm::Vec2, color: glm::U8Vec4) {
        let size = glm::length(&vec);
        simple_shape_renderer.send_rotated_rectangle(
            self.position,
            glm::vec2(0.5, 0.04) * size,
            PositionMode::CenterLeft,
            glm::vec2(-vec.y, vec.x),
            color, 2000, 0);
        
        simple_shape_renderer.send_symmetrical_triangle(
            self.position + vec * 0.6,
            self.position + vec * 0.5,
            size * 0.1,
            color, 2000, 0);
    }

    pub fn draw_compute_info(&self, font: &mut Font, simple_shape_renderer: &mut SimpleShapeRenderer) {
        self.instructions.send_draw(
            glm::vec2(1.0, 0.01),
            glm::vec2(0.98, 0.78), 
            PositionMode::BottomCenter,
            font, simple_shape_renderer,
            Some(self.memory.read_memory(PROGRAM_COUNTER_IDX) as usize),
            false,
        );

        self.memory.send_draw(
            glm::vec2(1.0, -0.05),
            glm::vec2(0.9, 0.7),
            PositionMode::TopCenter,
            font, simple_shape_renderer,
            4,
        );

        let code_background = glm::vec4(10, 10, 20, 255);
        simple_shape_renderer.send_rectangle(
            glm::vec2(1.0, 0.0),
            glm::vec2(1.0, 1.6),
            PositionMode::Center,
            code_background,
            50001, 1,
        );
    }


    pub fn draw(&self, simple_shape_renderer: &mut SimpleShapeRenderer) {
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
                self.position + weel_offset * WEEL_DISTANCE,
                glm::vec2(WEEL_DIAMETER, WEEL_DIAMETER),
                PositionMode::Center,
                glm::vec4(WEEL_COLOR[0], WEEL_COLOR[1], WEEL_COLOR[2], 255),
                WEELS_DEPTH, 0
            );
        }

        simple_shape_renderer.send_rectangle(
            self.position,
            glm::vec2(ROBOT_SIZE, ROBOT_SIZE),
            PositionMode::Center,
            glm::vec4(self.color.x, self.color.y, self.color.z, 255),
            ROBOT_DEPTH, 0
        );
    }















    pub fn calculate_acceleration(&mut self) -> glm::Vec2 {
        const PROPORTIONAL: f32 = 10.0;
        const DERIVATIVE: f32 = -6.0;

        let distance = self.target - self.position;
        let d = self.speed * DERIVATIVE;
        let p = distance * PROPORTIONAL;
        return p + d;
    }




























    pub fn movement_logic(&mut self, delta: f32, graph: &mut Graph) {
        let diff = glm::normalize(&(self.target - self.position));

        graph.wait();
        graph.push(0, glm::dot(&self.acc, &diff));
        graph.push(1, glm::dot(&self.drag, &diff));
        graph.push(2, glm::dot(&(self.target - self.position), &diff));
        graph.push(3, glm::dot(&self.speed, &diff));

        let sq = |x| x * x;

        if self.on_movement_instruction() {
            let target_accel_vec = self.calculate_acceleration();

            let accel_mag = glm::length(&target_accel_vec);

            let accel_adjust = accel_mag.min(MAX_ACCELLERATION);
            let target_accel_vec_applied = target_accel_vec / accel_mag * accel_adjust;

            let speed_val = glm::length(&self.speed);
            let norm_speed = self.speed / (speed_val + EPS);
            let drag_vector = -norm_speed * sq(speed_val) * DRAG;
            let all_acceleration = drag_vector + target_accel_vec_applied;

            self.position += self.speed * delta + all_acceleration * sq(delta) * 0.5;
            self.speed += all_acceleration * delta;

            self.drag = drag_vector;
            self.acc = target_accel_vec_applied;

        } else if self.speed == glm::vec2(0.0, 0.0) {
            return;
        
        } else {
            let speed_val = glm::length(&self.speed);
            let norm_speed = self.speed / (speed_val + EPS);
            let acc_vec = -norm_speed * MAX_ACCELLERATION;

            self.acc = glm::vec2(0.0, 0.0);
            self.drag = glm::vec2(0.0, 0.0);

            let time_to_stop = speed_val / MAX_ACCELLERATION;
            let virtual_delta = delta.min(time_to_stop);

            self.position += self.speed * virtual_delta - acc_vec * sq(virtual_delta) * 0.5;
            self.speed = glm::vec2(0.0, 0.0);
        }
    }
}