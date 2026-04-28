
use crate::{MainContext, game::{logic::{computer::MEMORY_SIZE, instruction::{Syscall, UnpackedInstruction}, robot::Robot}, render::{UI_BASE_DEPTH, dnd::Dnd, instruction_unit::{INSTRUCTION_HEIGHT, INSTRUCTION_WIDTH, InstructionUnit}}, scene::EventType}, graphics::renderer::{fonts::{TextAlign, TextRenderConfig}, positioning::{PositionMode, RenderBox}}, ui::{Ui, simple_area::SimpleArea}};
use nalgebra_glm as glm;

const BUTTON_AREA_WIDTH: f32 = 0.4;
const BUTTON_HEIGHT: f32 = 0.15;
const BUTTON_WIDTH: f32 = BUTTON_AREA_WIDTH - 4.0 * INFO_PADDING;
const BUTTON_PADDING: f32 = 0.02;
pub const INFO_WIDTH: f32 = 0.7;
pub const INFO_HEIGHT: f32 = 1.6;
pub const INFO_PADDING: f32 = 0.015;
pub const INFO_MARGIN: f32 = 0.02;

#[derive(Debug)]
pub struct RobotInfo {
    dnd: Dnd,
    instructions: Vec<InstructionUnit>,
    add_instruction_button_id: usize,
    pub robot_button_id: usize,
    pub info_area_id: usize,
}

impl RobotInfo {
    pub fn new(ui: &mut Ui, robot_button_id: usize, robot: &Robot) -> Self {
        let top_left = ui.get_ndc_render_box().get_top_left();
        let considered_instructions = robot.computer.instructions
            .iter()
            .filter(|instruction| instruction.unpack() != UnpackedInstruction::Sys(Syscall::Nop));

        let dnd = Dnd::new(
            top_left + Self::get_idx_default_offset(0),
            glm::vec2(INSTRUCTION_WIDTH, INSTRUCTION_HEIGHT),
            INFO_PADDING,
            considered_instructions.count(), ui);

        let instructions = robot.computer.instructions
            .iter()
            .filter(|instruction| instruction.unpack() != UnpackedInstruction::Sys(Syscall::Nop))
            .enumerate()
            .map(|(idx, instruction)| InstructionUnit::new(
                *instruction, ui,
                top_left + Self::get_idx_default_offset(idx),
                dnd.get_drag_box_id(idx),
            )).collect();

        Self {
            add_instruction_button_id: ui.simple_area_manager.push_simple_area(SimpleArea::new_button(
               Self::get_add_instruction_button_box(top_left), UI_BASE_DEPTH - 1)),
            dnd,
            instructions,
            robot_button_id,
            info_area_id: ui.simple_area_manager.push_simple_area(SimpleArea::new(
                Self::get_outer_area_render_box(top_left),
                UI_BASE_DEPTH,
            )),
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
        let top_left = ui.get_ndc_render_box().get_top_left();

        if id == self.add_instruction_button_id {
            if self.instructions.len() < MEMORY_SIZE {
                self.dnd.push(ui);
                self.instructions.push(InstructionUnit::new_blank(
                    ui,
                    top_left + Self::get_idx_default_offset(self.instructions.len()),
                    self.dnd.get_drag_box_id(self.dnd.len() - 1),
                ));
            }
        } else if let Some(instruction_to_remove) = self.instructions
            .iter()
            .enumerate()
            .find(|(_, instruction)| instruction.delete_button_id == id)
            .map(|(i, _)| i)
        {
            self.dnd.remove_original(instruction_to_remove);
            self.instructions.remove(instruction_to_remove);
        }
    }

    fn get_idx_drag_area(idx: usize, top_left: glm::Vec2) -> SimpleArea {
        SimpleArea::new_button(RenderBox::new_simple(
            top_left + Self::get_idx_default_offset(idx),
            PositionMode::TopLeft,
            glm::vec2(INSTRUCTION_WIDTH, INSTRUCTION_HEIGHT),
        ), UI_BASE_DEPTH - 1)
    }

    pub fn receive_move_event(&mut self, id: usize, movement: glm::Vec2) {
        self.dnd.receive_move_event(id, movement);
    }

    pub fn receive_release_event(&mut self, id: usize) {
        self.dnd.receive_release_event(id);
    }

    fn get_outer_area_render_box(top_left: glm::Vec2) -> RenderBox {
        RenderBox::new_simple(
            top_left + glm::vec2(INFO_MARGIN, -INFO_MARGIN),
            PositionMode::TopLeft,
            glm::vec2(INFO_WIDTH + BUTTON_AREA_WIDTH, INFO_HEIGHT),
        )
    }

    pub fn receive_hover_event(&mut self, event: &EventType) {
        for instruction in self.instructions.iter_mut() {
            instruction.receive_hover_event(event);
        }
    }

    pub fn send(&mut self, context: &mut MainContext) {
        let top_left = context.ui.get_ndc_render_box().get_top_left();
        self.dnd.top_left_position = top_left + Self::get_idx_default_offset(0);
        self.dnd.update(context.ui);
        context.ui.simple_area_manager.update_simple_area(self.info_area_id, SimpleArea::new(
            Self::get_outer_area_render_box(top_left),
            UI_BASE_DEPTH,
        ));

        let add_instruction_button_box = Self::get_add_instruction_button_box(top_left);
        context.ui.simple_area_manager.update_simple_area(
            self.add_instruction_button_id,
            SimpleArea::new_button(add_instruction_button_box, UI_BASE_DEPTH - 1),
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

        for (i, (bbox, is_grabbed)) in self.dnd.get_unit_positions().enumerate() {
            self.instructions[i].update(
                bbox.get_top_left(),
                context.ui,
            );

            self.instructions[i].send(context, is_grabbed, &bbox);
        }

        for i in 0..MEMORY_SIZE {
            let slot_box = Self::get_idx_drag_area(i, top_left).bounding_box;
            context.simple_shape_renderer.send_render_box(
                &slot_box,
                glm::vec4(18, 20, 30, 255),
                UI_BASE_DEPTH - 1, 1,
            );
        }

        context.simple_shape_renderer.send_render_box(
            &Self::get_outer_area_render_box(top_left),
            glm::vec4(15, 17, 26, 255),
            UI_BASE_DEPTH, 1,
        );
    }
}
