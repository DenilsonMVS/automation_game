
use nalgebra_glm as glm;

use crate::{game::instruction::Instruction, graphics::renderer::{fonts::{Font, TextRenderConfig}, positioning::PositionMode, simple_shape_renderer::SimpleShapeRenderer}};

const ZERO_IDX: usize = 0;
pub const PROGRAM_COUNTER_IDX: usize = 1;

#[derive(Debug)]
pub struct Memory<T: Copy + Default, const SIZE: usize> {
    memory: [T; SIZE],
}

impl<T: Copy + Default, const SIZE: usize> Memory<T, SIZE> {
    pub fn new() -> Self {
        Self {
            memory: [T::default(); SIZE]
        }
    }

    pub fn read_memory<ADDR: Into<usize>>(&self, address: ADDR) -> T {
        self.memory[address.into() & (SIZE - 1)]
    }

    pub fn write_memory<ADDR: Into<usize>>(&mut self, address: ADDR, value: T) {
        let address = address.into();
        if address != ZERO_IDX {
            self.memory[address & (SIZE - 1)] = value;
        }
    }

    pub fn write_instruction_memory<ADDR: Into<usize>>(&mut self, address: ADDR, value: T) {
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
        ).set_mode(PositionMode::TopLeft));
        font.add_text(&TextRenderConfig::new(
            &visualization[1],
            position + glm::vec2(glyph_space * 5.0, 0.0),
            line_height, code_colors[1],
        ).set_mode(PositionMode::TopLeft));
        font.add_text(&TextRenderConfig::new(
            &visualization[2],
            position + glm::vec2(glyph_space * 10.0, 0.0),
            line_height, code_colors[2],
        ).set_mode(PositionMode::TopLeft));
        font.add_text(&TextRenderConfig::new(
            &visualization[3],
            position + glm::vec2(glyph_space * 15.0, 0.0),
            line_height, code_colors[3],
        ).set_mode(PositionMode::TopLeft));   
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
        ).set_mode(PositionMode::TopLeft));
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
            ));
        }
    }
}
