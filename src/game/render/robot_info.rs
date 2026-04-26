
use crate::{MainContext, game::{logic::{computer::MEMORY_SIZE, instruction::{Syscall, UnpackedInstruction}, memory::GenericMemory, robot::Robot}, render::UI_BASE_DEPTH}, graphics::renderer::{fonts::{TextAlign, TextRenderConfig}, positioning::{PositionMode, RenderBox}}, ui::{Ui, drag_area::DragArea}};
use nalgebra_glm as glm;

const INFO_WIDTH: f32 = 0.8;
const INFO_HEIGHT: f32 = 1.6;
const INFO_PADDING: f32 = 0.015;
const INFO_MARGIN: f32 = 0.02;
const INSTRUCTION_WIDTH: f32 = INFO_WIDTH - 2.0 * INFO_PADDING;
const INSTRUCTION_HEIGHT: f32 = (INFO_HEIGHT - INFO_PADDING * (MEMORY_SIZE + 1) as f32) / MEMORY_SIZE as f32;
const INSTRUCTION_PADDING: f32 = 0.01;

#[derive(Debug, Default)]
pub struct RobotInfo {
    instruction_grab_boxes: Vec<(usize, glm::Vec2)>,
    pub robot_button_id: usize,
}

impl RobotInfo {
    pub fn new(ui: &mut Ui, robot_button_id: usize, robot: &Robot) -> Self {
        let top_left = ui.get_ndc_render_box().get_top_left();
        let instruction_grab_boxes = robot.computer.instructions
            .iter()
            .filter(|inst| inst.unpack() != UnpackedInstruction::Sys(Syscall::Nop))
            .enumerate()
            .map(|(i, _)| (
                ui.drag_area_manager.push_drag_area(Self::get_idx_drag_area(i, top_left)),
                glm::vec2(0.0, 0.0),
            ))
            .collect();

        Self {
            instruction_grab_boxes,
            robot_button_id,
        }
    }

    fn get_idx_default_offset(idx: usize) -> glm::Vec2 {
        glm::vec2(
            INFO_MARGIN + INFO_PADDING,
            -(INFO_MARGIN + INFO_PADDING + (INSTRUCTION_HEIGHT + INFO_PADDING) * idx as f32),
        )
    }

    fn get_idx_drag_area(idx: usize, top_left: glm::Vec2) -> DragArea {
        DragArea::new(RenderBox::new_simple(
            top_left + Self::get_idx_default_offset(idx),
            PositionMode::TopLeft,
            glm::vec2(INSTRUCTION_WIDTH, INSTRUCTION_HEIGHT),
        ))
    }

    pub fn receive_move_event(&mut self, id: usize, movement: glm::Vec2) {
        if let Some((_, offset)) = self.instruction_grab_boxes
            .iter_mut()
            .find(|(instruction_id, _)| *instruction_id == id)
        {
            offset.y += movement.y;
        }
    }

    pub fn receive_release_event(&mut self, id: usize) {
        if let Some((_, offset)) = self.instruction_grab_boxes
            .iter_mut()
            .find(|(instruction_id, _)| *instruction_id == id)
        {
            offset.y = 0.0;
        }
    }

    fn get_grabbed_idx(&self) -> Option<usize> {
        self.instruction_grab_boxes
            .iter()
            .enumerate()
            .find(|(_, (_, offset))| *offset != glm::vec2(0.0, 0.0))
            .map(|(idx, _)| idx)
    }

    fn get_visualization_iter(&self) -> impl Iterator<Item = usize> + '_ {
        let len = self.instruction_grab_boxes.len();
        
        // Calculamos o grabbed e o target uma única vez fora do iterador
        let params = self.get_grabbed_idx().map(|g| {
            let space = INSTRUCTION_HEIGHT + INFO_PADDING;
            let offset_y = -self.instruction_grab_boxes[g].1.y;
            let shift = (offset_y / space).round() as i32;
            let t = (g as i32 + shift).clamp(0, (len - 1) as i32) as usize;
            (g, t)
        });

        (0..len).map(move |i| {
            let Some((grabbed, target)) = params else {
                return i; // Se não está segurando nada, a ordem é 1:1
            };

            if i == target {
                grabbed // Na posição alvo, mostre o item segurado
            } else if grabbed < target && i >= grabbed && i < target {
                i + 1 // Shift para cima se o item segurado desceu
            } else if grabbed > target && i > target && i <= grabbed {
                i - 1 // Shift para baixo se o item segurado subiu
            } else {
                i // Posição não afetada
            }
        })
    }

    pub fn send(&self, context: &mut MainContext, robot: &Robot) {
        let top_left = context.ui.get_ndc_render_box().get_top_left();
        let grabbed_idx = self.get_grabbed_idx();

        for (i, (id, _)) in self.instruction_grab_boxes.iter().enumerate() {
            context.ui.drag_area_manager.update_drag_area(
                *id, Self::get_idx_drag_area(i, top_left));
        }

        if let Some(g_idx) = grabbed_idx {
            let instruction = robot.computer.instructions.read(g_idx).unpack();
            if instruction != UnpackedInstruction::Sys(Syscall::Nop) {
                let (_, offset) = self.instruction_grab_boxes[g_idx];
                let bounding_box = Self::get_idx_drag_area(g_idx, top_left)
                    .bounding_box
                    .apply_offset(offset);

                self.render_instruction_at(context, &instruction, &bounding_box, true);
            }
        }

        for (visual_row, original_idx) in self.get_visualization_iter().enumerate() {
            if Some(original_idx) == grabbed_idx { continue; }

            let instruction = robot.computer.instructions.read(original_idx).unpack();
            if instruction == UnpackedInstruction::Sys(Syscall::Nop) { continue; }

            let bounding_box = Self::get_idx_drag_area(visual_row, top_left).bounding_box;
            self.render_instruction_at(context, &instruction, &bounding_box, false);
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
            glm::vec2(INFO_WIDTH, INFO_HEIGHT),
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
