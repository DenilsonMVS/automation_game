use crate::{MainContext, game::{logic::instruction::{InstructionId, UnpackedInstruction}, render::{colored_text::{ColoredTextButton, ColoredTextView}, robot_info::{get_list_bgcolor, get_secondary_background_color}}}, graphics::renderer::{fonts::Font, positioning::{PositionMode, RenderBox}}, ui::{dnd::HasId, menu::Menu, rendered_button::{RenderedButton, RenderedButtonContext}, simple_area_manager::AreaId, ui_element::UiElement}};
use nalgebra_glm as glm;

const PADDING: f32 = 0.01;

fn get_delete_button_color() -> glm::U8Vec4 {
    glm::vec4(200, 50, 60, 255)
}

#[derive(Debug)]
pub struct InstructionEditor {
    id: AreaId,
    bbox: RenderBox,
    depth: u16,
    opcode_menu: Menu<ColoredTextButton, ColoredTextView>,
    delete_button: RenderedButton,
}

impl InstructionEditor {
    pub fn new() -> Self {
        let opcodes = (0..16)
            .map(InstructionId::from_u16)
            .map(|inst| format!("{}", inst))
            .map(|opcode_str| ColoredTextButton::new(
                opcode_str,
                UnpackedInstruction::get_mnemonic_color(),
                get_secondary_background_color(),
            ))
            .collect();
        Self {
            id: AreaId::default(),
            bbox: Default::default(),
            depth: 0,
            opcode_menu: Menu::new(opcodes).with_gap(PADDING).with_open_bgcolor(get_list_bgcolor()),
            delete_button: RenderedButton::new(get_delete_button_color()),
        }
    }

    fn get_delete_button_bbox(bbox: &RenderBox) -> RenderBox {
        let height = bbox.get_size().y - PADDING * 2.0;
        RenderBox::new_simple(
            bbox.get_top_left() + glm::vec2(PADDING, -PADDING),
            PositionMode::TopLeft,
            glm::vec2(height, height),
        )
    }

    fn get_opcode_bbox(bbox: &RenderBox, font: &Font) -> RenderBox {
        let delete_button_bbox = Self::get_delete_button_bbox(bbox);
        let height = delete_button_bbox.get_size().y;
        let amount_chars = 4.0;
        let char_width = font.get_monospace_width(height);
        RenderBox::new_simple(
            delete_button_bbox.get_top_right() + glm::vec2(PADDING * 2.0, 0.0),
            PositionMode::TopLeft,
            glm::vec2(char_width * amount_chars, height),
        )
    }

    pub fn get_delete_button_id(&self) -> AreaId {
        self.delete_button.id
    }
}

impl HasId for InstructionEditor {
    fn get_id(&self) -> AreaId {
        self.id
    }

    fn update_id(&mut self, id: AreaId) {
        self.id = id;
    }
}

impl<'a, 'b> UiElement<MainContext<'a, 'b>, MainContext<'a, 'b>, ()> for InstructionEditor {
    fn update(&mut self, bbox: &RenderBox, depth: u16, context: &mut MainContext<'a, 'b>) {
        self.bbox = *bbox;
        self.depth = depth;

        self.delete_button.update(&Self::get_delete_button_bbox(bbox), depth - 1, context.ui);
        self.opcode_menu.update(&Self::get_opcode_bbox(bbox, context.font), depth - 1, context.ui);
    }

    fn send(&mut self, context: &mut MainContext) {
        let mut button_context = RenderedButtonContext {
            font: context.font,
            simple_shape_renderer: context.simple_shape_renderer,
        };
        self.delete_button.send(&mut button_context);

        context.simple_shape_renderer.send_render_box(&self.bbox, get_secondary_background_color(), self.depth, None);
        self.opcode_menu.send(context);
    }
}
