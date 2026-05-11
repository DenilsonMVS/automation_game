use winit::window::CursorIcon;

use crate::{MainContext, game::{logic::instruction::Instruction, render::{instruction_editor, robot_info::get_secondary_background_color}}, graphics::renderer::{fonts::TextRenderConfig, positioning::{PositionMode, RenderBox}}, ui::{rendered_area::RenderedArea, ui_element::UiElement}};

#[derive(Debug)]
pub struct InstructionView {
    rendered_area: RenderedArea,
    instruction: Instruction,
}

impl InstructionView {
    pub fn new(instruction: Instruction) -> Self {
        Self {
            rendered_area: RenderedArea::new(CursorIcon::Default, get_secondary_background_color()),
            instruction,
        }
    }

    pub fn get_instruction(&self) -> Instruction {
        self.instruction
    }

    pub fn update_execution_state(&mut self, being_executed: bool) {
        self.rendered_area.color = if being_executed {
            get_secondary_background_color().map(|x| x.saturating_add(25))
        } else {
            get_secondary_background_color()
        };
    }
}

impl<'a, 'b> UiElement<MainContext<'a, 'b>, MainContext<'a, 'b>, ()> for InstructionView {
    fn update(&mut self, bbox: &RenderBox, depth: u16, context: &mut MainContext) {
        self.rendered_area.update(bbox, depth, context.ui);
    }

    fn send(&mut self, mc: &mut MainContext) {
        if self.instruction.unpack() == Instruction::default().unpack() {
            return;
        }

        self.rendered_area.send(&mut (mc.simple_shape_renderer, mc.groups));

        let inner_box = self.rendered_area.bbox.cut(instruction_editor::PADDING);
        let unpacked = self.instruction.unpack();
        let instruction_text = unpacked.format_assembly();
        let colors = unpacked.format_with_colors(&instruction_text);
        mc.font.add_text(&TextRenderConfig::new(
            &colors,
            inner_box.get_top_left(),
            inner_box.get_size().y,
        ).set_mode(PositionMode::TopLeft).set_depth(self.rendered_area.depth - 1), mc.groups);
    }
}
