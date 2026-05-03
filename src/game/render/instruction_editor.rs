use crate::{game::render::robot_info::get_secondary_background_color, graphics::renderer::{positioning::RenderBox, simple_shape_renderer::SimpleShapeRenderer}, ui::{Ui, dnd::{HasId, HasUi}, simple_area_manager::AreaId, ui_element::UiElement}};


#[derive(Debug, Default)]
pub struct InstructionEditor {
    id: AreaId,
    bbox: RenderBox,
    depth: u16,
}

impl InstructionEditor {
    pub fn new() -> Self {
        Self { id: AreaId::default(), ..Default::default() }
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

impl HasUi for Ui {
    fn get_ui(&mut self) -> &mut Ui {
        self
    }
}

impl<'a> UiElement<Ui, SimpleShapeRenderer<'a>, ()> for InstructionEditor {
    fn update(&mut self, bbox: &RenderBox, depth: u16, _ui: &mut Ui) {
        self.bbox = *bbox;
        self.depth = depth;
    }

    fn send(&mut self, simple_shape_renderer: &mut SimpleShapeRenderer<'a>) {
        simple_shape_renderer.send_render_box(&self.bbox, get_secondary_background_color(), self.depth, 1);
    }
}
