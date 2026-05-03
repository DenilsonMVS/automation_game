use winit::window::CursorIcon;
use nalgebra_glm as glm;
use crate::{game::{logic::computer::MEMORY_SIZE, render::{INFO_HEIGHT, INFO_WIDTH, instruction_editor::InstructionEditor}}, graphics::renderer::positioning::{PositionMode, RenderBox}, ui::{Ui, dnd::Dnd, rendered_area::RenderedArea, rendered_button::{RenderedButton, RenderedButtonContext}, ui_element::UiElement}};

pub fn get_background_color() -> glm::U8Vec4 {
    glm::vec4(8, 17, 25, 255)
}

pub fn get_secondary_background_color() -> glm::U8Vec4 {
    glm::vec4(38, 47, 55, 255)
}

pub fn get_add_instruction_button_bgcolor() -> glm::U8Vec4 {
    glm::vec4(0x5C, 0x6B, 0xC0, 255)
}

pub fn get_add_instruction_button_color() -> glm::U8Vec4 {
    glm::vec4(255, 255, 255, 255)
}

const CONTAINER_PADDING: f32 = 0.02;
const BUTTTON_AREA_WIDTH: f32 = 0.4;
const BUTTON_HEIGHT: f32 = 0.15;
const BUTTON_WIDTH: f32 = BUTTTON_AREA_WIDTH - 2.0 * CONTAINER_PADDING;
const INSTRUCTION_WIDTH: f32 = INFO_WIDTH - BUTTTON_AREA_WIDTH - 2.0 * CONTAINER_PADDING;
const INSTRUCTION_HEIGHT: f32 = (INFO_HEIGHT - (MEMORY_SIZE + 1) as f32 * CONTAINER_PADDING) / MEMORY_SIZE as f32;

#[derive(Debug)]
pub struct RobotInfo {
    pub robot_idx: usize,
    outer_container: RenderedArea,
    add_instruction_button: RenderedButton,
    instructions: Dnd<InstructionEditor>,
}

impl RobotInfo {
    pub fn new(robot_idx: usize) -> Self {
        let mut dnd = Dnd::new(glm::vec2(INSTRUCTION_WIDTH, INSTRUCTION_HEIGHT))
            .with_gap(CONTAINER_PADDING);
        for _ in 0..8 {
            dnd.push(InstructionEditor::new());
        }

        Self {
            robot_idx,
            outer_container: RenderedArea::new(CursorIcon::Default, get_background_color()),
            add_instruction_button: RenderedButton::new(get_add_instruction_button_bgcolor())
                .with_content("ADD", get_add_instruction_button_color()),
            instructions: dnd,
        }
    }

    fn get_add_instruction_button_container(bbox: &RenderBox) -> RenderBox {
        RenderBox::new_simple(
            bbox.get_top_right() + glm::vec2(-CONTAINER_PADDING, -CONTAINER_PADDING),
            PositionMode::TopRight,
            glm::vec2(BUTTON_WIDTH, BUTTON_HEIGHT),
        )
    }

    fn get_dnd_bbox(bbox: &RenderBox) -> RenderBox {
        RenderBox::new_simple(
            bbox.get_top_left() + glm::vec2(CONTAINER_PADDING, -CONTAINER_PADDING),
            PositionMode::TopLeft,
            glm::vec2(INFO_WIDTH - BUTTON_WIDTH - 2.0 * CONTAINER_PADDING, INFO_HEIGHT - 2.0 * CONTAINER_PADDING),
        )
    }
}

impl<'a, 'b> UiElement<Ui, RenderedButtonContext<'a, 'b>, ()> for RobotInfo {
    fn update(&mut self, bbox: &RenderBox, depth: u16, ui: &mut Ui) {
        self.outer_container.update(bbox, depth, ui);
        self.add_instruction_button.update(
            &Self::get_add_instruction_button_container(bbox),
            depth - 1, ui,
        );
        self.instructions.update(&Self::get_dnd_bbox(bbox), depth - 1, ui);
    }

    fn send(&mut self, context: &mut RenderedButtonContext<'a, 'b>) {
        self.instructions.send(context.simple_shape_renderer);
        self.add_instruction_button.send(context);
        self.outer_container.send(context.simple_shape_renderer);
    }
}
