use crate::{game::logic::top_level::{computer::MEMORY_SIZE, instruction::UnpackedInstruction, memory::PROGRAM_COUNTER_IDX, robot::RobotLogic}, graphics::renderer::{Renderer, fonts::{TextAlign, TextRenderConfig, font::Font}, positioning::PositionMode}, ui::{Ui, UiState}};
use nalgebra_glm as glm;

pub mod instruction;

const UI_BASE_DEPTH: u16 = 50000;
const LINES_SPACING: f32 = 0.01;
const ITEM_HEIGHT: f32 = 0.06;
const ITEM_PADDING: f32 = 0.005;
const ITEM_WIDTH: f32 = 0.5;

pub fn get_background_color() -> glm::U8Vec4 {
    glm::vec4(8, 17, 25, 255)
}

pub fn get_secondary_background_color() -> glm::U8Vec4 {
    glm::vec4(38, 47, 55, 255)
}

pub fn get_instruction_being_executed_color() -> glm::U8Vec4 {
    glm::vec4(38, 47, 100, 255)
}

pub fn get_reg_separator_color() -> glm::U8Vec4 {
    glm::vec4(188, 197, 55, 255)
}

pub struct RobotInfo {
    robot_idx: usize,
}

impl RobotInfo {
    pub fn new(robot_idx: usize) -> Self {
        Self {
            robot_idx,
        }
    }

    pub fn get_robot_idx(&self) -> usize {
        self.robot_idx
    }

    pub fn update(
        &mut self,
        ui: &mut Ui,
        renderer: &mut Renderer,
        font: &mut Font,
        watched_robots: &[RobotLogic],
    ) {
        let screen_bbox = ui.get_screen_bbox();

        let bbox = screen_bbox.sub_box(
            glm::vec2(0.015, -0.015),
            glm::vec2(1.0, 1.6),
            PositionMode::TopLeft,
            PositionMode::TopLeft,
        );
        renderer.simple_shape_renderer.send_render_box(
            &bbox,
            get_background_color(),
            UI_BASE_DEPTH, 1,
        );

        let instructions = watched_robots.get(0).map(|r| r.computer.get());
        let pc_value = watched_robots.get(0).map(|r| r.computer.get_data()[PROGRAM_COUNTER_IDX] % MEMORY_SIZE as u8);
        for i in 0..MEMORY_SIZE {
            let pos = (i as f32) * (ITEM_HEIGHT + LINES_SPACING) + LINES_SPACING;
            let bbox = bbox.sub_box(
                glm::vec2(LINES_SPACING, -pos),
                glm::vec2(ITEM_WIDTH, ITEM_HEIGHT),
                PositionMode::TopLeft,
                PositionMode::TopLeft,
            );
            let depth = UI_BASE_DEPTH - 1;

            let block_color = if Some(i as u8) == pc_value {
                get_instruction_being_executed_color()
            } else {
                get_secondary_background_color()
            };
            
            renderer.simple_shape_renderer.send_render_box(
                &bbox, block_color,
                depth, 1,
            );
        
            if let Some(instructions) = instructions {
                let mut ui_state = UiState { renderer, ui, font };
                instructions[i].update(&mut ui_state, &bbox, depth);
            }
        }

        if let Some(registers) = watched_robots.get(0).map(|r| r.computer.get_data()) {
            const SPACING: f32 = 0.005;
            const REG_BOX_WIDTH: f32 = 0.09;
            const REG_BOX_HEIGHT: f32 = 0.06;
            const OUTER_WIDTH: f32 = (MEMORY_SIZE / 2) as f32 * (REG_BOX_WIDTH + SPACING) + SPACING;
            const OUTER_HEIGHT: f32 = (MEMORY_SIZE / 8) as f32 * (REG_BOX_HEIGHT + SPACING) + SPACING;
            
            let bbox = bbox.sub_box(
                glm::vec2(SPACING, SPACING),
                glm::vec2(OUTER_WIDTH, OUTER_HEIGHT),
                PositionMode::BottomLeft,
                PositionMode::BottomLeft,
            );
            let depth = UI_BASE_DEPTH - 1;
            
            renderer.simple_shape_renderer.send_render_box(
                &bbox,
                get_reg_separator_color(),
                depth, 1,
            );

            for i in 0..2 {
                for j in 0..8 {
                    let x_offset = (j as f32) * (REG_BOX_WIDTH + SPACING) + SPACING;
                    let y_offset = (i as f32) * (REG_BOX_HEIGHT + SPACING) + SPACING;
                    let offset = glm::vec2(x_offset, -y_offset);
                    let unit_size = glm::vec2(REG_BOX_WIDTH, REG_BOX_HEIGHT);

                    let bbox = bbox.sub_box(
                        offset, unit_size,
                        PositionMode::TopLeft, PositionMode::TopLeft,
                    );
                    let depth = depth - 1;

                    renderer.simple_shape_renderer.send_render_box(
                        &bbox,
                        get_secondary_background_color(),
                        depth, 1,
                    );


                    fn u8_to_hex(val: u8) -> [u8; 2] {
                        const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
                        [
                            HEX_CHARS[(val >> 4) as usize],
                            HEX_CHARS[(val & 0x0f) as usize],
                        ]
                    }
                    let bytes = u8_to_hex(registers[i * 8 + j]);
                    let hex = unsafe { std::str::from_utf8_unchecked(&bytes) };

                    renderer.font_renderer.push(&TextRenderConfig::new(
                        &[(&hex, UnpackedInstruction::get_literal_color())],
                        bbox.get_center(),
                        bbox.get_size().y - SPACING * 2.0,
                    ).with_align(TextAlign::Center), font, 1);
                }
            }
        }
    }
}
