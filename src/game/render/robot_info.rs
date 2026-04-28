
use crate::{MainContext, game::{logic::{computer::MEMORY_SIZE, instruction::{Instruction, Syscall, UnpackedInstruction}, robot::Robot}, render::{UI_BASE_DEPTH, dnd::Dnd}}, graphics::renderer::{fonts::{TextAlign, TextRenderConfig}, positioning::{PositionMode, RenderBox}}, ui::{Ui, button_area::ButtonArea, drag_area::DragArea}};
use nalgebra_glm as glm;

const BUTTON_AREA_WIDTH: f32 = 0.4;
const BUTTON_HEIGHT: f32 = 0.15;
const BUTTON_WIDTH: f32 = BUTTON_AREA_WIDTH - 4.0 * INFO_PADDING;
const BUTTON_PADDING: f32 = 0.02;
const INFO_WIDTH: f32 = 0.7;
const INFO_HEIGHT: f32 = 1.6;
const INFO_PADDING: f32 = 0.015;
const INFO_MARGIN: f32 = 0.02;
const INSTRUCTION_WIDTH: f32 = INFO_WIDTH - 2.0 * INFO_PADDING;
const INSTRUCTION_HEIGHT: f32 = (INFO_HEIGHT - INFO_PADDING * (MEMORY_SIZE + 1) as f32) / MEMORY_SIZE as f32;
const INSTRUCTION_PADDING: f32 = 0.01;

#[derive(Debug)]
pub enum InstructionConfig {
    None,
    FinishedInstruction(Instruction),
    UnfinishedInstruction(Option<u8>, Option<u8>, Option<u8>, Option<u8>),
}

#[derive(Debug)]
pub struct RobotInfo {
    dnd: Dnd,
    instructions: Vec<InstructionConfig>,
    add_instruction_button_id: usize,
    pub robot_button_id: usize,
}

impl RobotInfo {
    pub fn new(ui: &mut Ui, robot_button_id: usize, robot: &Robot) -> Self {
        let top_left = ui.get_ndc_render_box().get_top_left();
        let instructions: Vec<InstructionConfig> = robot.computer.instructions
            .iter()
            .filter_map(|instruction| {
                if instruction.unpack() == UnpackedInstruction::Sys(Syscall::Nop) {
                    None
                } else {
                    Some(InstructionConfig::FinishedInstruction(*instruction))
                }
            }).collect();

        Self {
            add_instruction_button_id: ui.button_manager.push_buton_area(ButtonArea::new(
               Self::get_add_instruction_button_box(top_left))),
            dnd: Dnd::new(
                top_left + Self::get_idx_default_offset(0),
                glm::vec2(INSTRUCTION_WIDTH, INSTRUCTION_HEIGHT),
                INFO_PADDING,
                instructions.len(), ui),
            instructions,
            robot_button_id,
        }
    }

    fn get_add_instruction_button_box(top_left: glm::Vec2) -> RenderBox {
        RenderBox::new_simple(
            top_left + glm::vec2(INFO_WIDTH + INFO_MARGIN + INFO_PADDING * 2.0, -INFO_MARGIN - INFO_PADDING),
            PositionMode::TopLeft,
            glm::vec2(BUTTON_WIDTH, BUTTON_HEIGHT),
        )
    }

    fn get_idx_default_offset(idx: usize) -> glm::Vec2 {
        glm::vec2(
            INFO_MARGIN + INFO_PADDING,
            -(INFO_MARGIN + INFO_PADDING + (INSTRUCTION_HEIGHT + INFO_PADDING) * idx as f32),
        )
    }

    pub fn receive_button_click(&mut self, id: usize, ui: &mut Ui) {
        if id != self.add_instruction_button_id {
            return;
        }

        if self.instructions.len() < MEMORY_SIZE {
            self.instructions.push(InstructionConfig::UnfinishedInstruction(None, None, None, None));
            self.dnd.push(ui);
        }
    }

    fn get_idx_drag_area(idx: usize, top_left: glm::Vec2) -> DragArea {
        DragArea::new(RenderBox::new_simple(
            top_left + Self::get_idx_default_offset(idx),
            PositionMode::TopLeft,
            glm::vec2(INSTRUCTION_WIDTH, INSTRUCTION_HEIGHT),
        ))
    }

    pub fn receive_move_event(&mut self, id: usize, movement: glm::Vec2) {
        self.dnd.receive_move_event(id, movement);
    }

    pub fn receive_release_event(&mut self, id: usize) {
        self.dnd.receive_release_event(id);
    }

    pub fn send(&mut self, context: &mut MainContext) {
        let top_left = context.ui.get_ndc_render_box().get_top_left();
        self.dnd.top_left_position = top_left + Self::get_idx_default_offset(0);
        self.dnd.update(context.ui);

        let add_instruction_button_box = Self::get_add_instruction_button_box(top_left);
        context.ui.button_manager.update_button_area(
            self.add_instruction_button_id,
            ButtonArea::new(add_instruction_button_box),
        );
        context.simple_shape_renderer.send_render_box(
            &add_instruction_button_box,
            glm::vec4(0x5C, 0x6B, 0xC0, 255),
            UI_BASE_DEPTH - 1, 1,
        );
        context.font.add_text(
            &TextRenderConfig::new(
                &[("ADD", glm::vec4(255, 255, 255, 255))],
                add_instruction_button_box.get_top_left() + glm::vec2(BUTTON_PADDING, -BUTTON_PADDING),
                BUTTON_HEIGHT - BUTTON_PADDING * 2.0,
            )
            .set_align(TextAlign::Center)
            .set_depth(UI_BASE_DEPTH - 2)
            .set_max_line_width(BUTTON_WIDTH - BUTTON_PADDING * 2.0)
            .set_mode(PositionMode::TopLeft)
        );

        for (i, (bounding_box, is_grabbed)) in self.dnd.get_unit_positions().enumerate() {
            match self.instructions[i] {
                InstructionConfig::FinishedInstruction(instruction) => {
                    self.render_instruction_at(
                        context,
                        &instruction.unpack(),
                        &bounding_box,
                        is_grabbed,
                    );
                },
                InstructionConfig::UnfinishedInstruction(_, _, _, _) => {
                    context.simple_shape_renderer.send_render_box(
                        &bounding_box,
                        glm::vec4(25, 28, 39, 255),
                        UI_BASE_DEPTH - 2, 1,
                    );
                },
                _ => {},
            };
        }

        for i in 0..MEMORY_SIZE {
            let slot_box = Self::get_idx_drag_area(i, top_left).bounding_box;
            context.simple_shape_renderer.send_render_box(
                &slot_box,
                glm::vec4(18, 20, 30, 255),
                UI_BASE_DEPTH - 1, 1,
            );
        }

        context.simple_shape_renderer.send_rectangle(
            top_left + glm::vec2(INFO_MARGIN, -INFO_MARGIN),
            glm::vec2(INFO_WIDTH + BUTTON_AREA_WIDTH, INFO_HEIGHT),
            PositionMode::TopLeft,
            glm::vec4(15, 17, 26, 255),
            UI_BASE_DEPTH, 1,
        );
    }

    fn render_instruction_at(&self, context: &mut MainContext, inst: &UnpackedInstruction, bbox: &RenderBox, is_grabbed: bool) {
        let extra_depth = if is_grabbed { 10 } else { 0 };
        
        context.simple_shape_renderer.send_render_box(
            bbox,
            glm::vec4(25, 28, 39, 255),
            UI_BASE_DEPTH - 2 - extra_depth, 1,
        );

        let text_box = bbox.cut(INSTRUCTION_PADDING);
        let formatted_text = inst.format_assembly();
        let color_formatted = inst.format_with_colors(&formatted_text);

        context.font.add_text(&TextRenderConfig::new(
            &color_formatted,
            text_box.get_top_left(),
            text_box.get_size().y)
            .set_mode(PositionMode::TopLeft)
            .set_align(TextAlign::Left)
            .set_depth(UI_BASE_DEPTH - 3 - extra_depth),
        );
    }
}
