use crate::{MainContext, game::{logic::{computer::MEMORY_SIZE, instruction::Instruction}, render::{UI_BASE_DEPTH, robot_info::{INFO_HEIGHT, INFO_PADDING, INFO_WIDTH}}, scene::EventType}, graphics::renderer::{fonts::{TextAlign, TextRenderConfig}, positioning::{PositionMode, RenderBox}}, ui::{Ui, simple_area::SimpleArea}};
use nalgebra_glm as glm;

pub const INSTRUCTION_WIDTH: f32 = INFO_WIDTH - 2.0 * INFO_PADDING;
pub const INSTRUCTION_HEIGHT: f32 = (INFO_HEIGHT - INFO_PADDING * (MEMORY_SIZE + 1) as f32) / MEMORY_SIZE as f32;
const INSTRUCTION_PADDING: f32 = 0.01;
const DELETE_BUTTON_SIZE: f32 = INSTRUCTION_HEIGHT - 2.0 * INSTRUCTION_PADDING;

#[derive(Debug)]
pub enum InstructionConfig {
    FinishedInstruction(Instruction),
    UnfinishedInstruction(Option<u8>, Option<u8>, Option<u8>, Option<u8>),
}

#[derive(Debug)]
pub struct InstructionUnit {
    config: InstructionConfig,
    pub delete_button_id: usize,
    drag_area_id: usize,
    hovering_instruction: bool,
    hovering_delete_button: bool,
}

impl InstructionUnit {
    pub fn new(instruction: Instruction, ui: &mut Ui, unit_top_left_position: glm::Vec2, drag_area_id: usize) -> Self {
        Self {
            config: InstructionConfig::FinishedInstruction(instruction),
            delete_button_id: ui.simple_area_manager.push_simple_area(SimpleArea::new_button(
                Self::get_delete_button_box(unit_top_left_position),
                UI_BASE_DEPTH - 2,
            )),
            hovering_delete_button: false,
            hovering_instruction: false,
            drag_area_id,
        }
    }

    pub fn new_blank(ui: &mut Ui, unit_top_left_position: glm::Vec2, drag_area_id: usize) -> Self {
        Self {
            config: InstructionConfig::UnfinishedInstruction(None, None, None, None),
            delete_button_id: ui.simple_area_manager.push_simple_area(SimpleArea::new_button(
                Self::get_delete_button_box(unit_top_left_position),
                UI_BASE_DEPTH - 2,
            )),
            hovering_delete_button: false,
            hovering_instruction: false,
            drag_area_id,
        }
    }

    pub fn receive_hover_event(&mut self, event: &EventType) {
        match event {
            EventType::HoverDragArea(id) => {
                if *id == self.drag_area_id {
                    self.hovering_instruction = true;
                }
            },
            EventType::HoverSimpleArea(id) => {
                if *id == self.delete_button_id {
                    self.hovering_delete_button = true;
                }
            },
            _ => {}
        }
    }

    pub fn update(&mut self, unit_top_left_position: glm::Vec2, ui: &mut Ui) {
        ui.simple_area_manager.update_simple_area(
            self.delete_button_id,
            SimpleArea::new_button(
                Self::get_delete_button_box(unit_top_left_position),
                UI_BASE_DEPTH - 2,
            ),
        );
    }

    pub fn send(
        &mut self,
        context: &mut MainContext,
        is_grabbed: bool,
        bbox: &RenderBox,
    ) {
        let extra_depth = if is_grabbed { 10 } else { 0 };
        let unit_top_left_position = bbox.get_top_left();

        let delete_button_color = if self.hovering_delete_button {
            glm::vec4(230, 80, 80, 255)
        } else if self.hovering_instruction {
            glm::vec4(200, 50, 50, 255)
        } else {
            glm::vec4(5, 8, 19, 255)
        };

        self.hovering_delete_button = false;
        self.hovering_instruction = false;

        let delete_button_bbox = Self::get_delete_button_box(unit_top_left_position);
        context.simple_shape_renderer.send_render_box(
            &delete_button_bbox,
            delete_button_color,
            UI_BASE_DEPTH - 3 - extra_depth,
            1,
        );

        context.simple_shape_renderer.send_render_box(
            &bbox,
            glm::vec4(25, 28, 39, 255),
            UI_BASE_DEPTH - 2 - extra_depth, 1,
        );

        if let InstructionConfig::FinishedInstruction(inst) = self.config {
            let inst = inst.unpack();
            let text_box = delete_button_bbox.apply_offset(glm::vec2(DELETE_BUTTON_SIZE + INSTRUCTION_PADDING * 2.0, 0.0));
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

    fn get_delete_button_box(unit_top_left_position: glm::Vec2) -> RenderBox {
        RenderBox::new_simple(
            unit_top_left_position + glm::vec2(INSTRUCTION_PADDING, -INSTRUCTION_PADDING),
            PositionMode::TopLeft,
            glm::vec2(DELETE_BUTTON_SIZE, DELETE_BUTTON_SIZE),
        )
    }
}
