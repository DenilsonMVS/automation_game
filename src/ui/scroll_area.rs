use crate::{graphics::renderer::{positioning::RenderBox, scissor_group::ScissorStack, simple_shape_renderer::SimpleShapeRenderer}, ui::{Ui, simple_area::SimpleArea, simple_area_manager::AreaId, ui_element::UiElement}};
use nalgebra_glm as glm;

const SCROLL_SENSITIVITY: f32 = 0.02;

#[derive(Debug)]
pub struct ScrollArea {
    bbox: RenderBox,
    depth: u16,
    pub max_offset: f32,
    pub offset: f32,
    pub bgcolor: glm::U8Vec4,
    area_id: AreaId,
}

impl ScrollArea {
    pub fn new(bgcolor: glm::U8Vec4) -> Self {
        Self { bbox: Default::default(), depth: 0, offset: 0.0, max_offset: 0.0, bgcolor, area_id: Default::default() }
    }

    pub fn push_scissor_context(&self, groups: &mut ScissorStack) {
        groups.push(self.bbox);
    }

    pub fn apply_offset_to_box(&self, bbox: &RenderBox) -> RenderBox {
        bbox.apply_offset(glm::vec2(0.0, self.offset))
    }

    pub fn get_bbox(&self) -> &RenderBox {
        &self.bbox
    }
}

impl<'a, 'b> UiElement<Ui, (&mut SimpleShapeRenderer<'a>, &mut ScissorStack<'a>), ()> for ScrollArea {
    fn update(&mut self, bbox: &RenderBox, depth: u16, ui: &mut Ui) {
        self.bbox = *bbox;
        self.depth = depth;

        if ui.events.scroll == self.area_id {
            self.offset -= ui.scroll * SCROLL_SENSITIVITY;
            self.offset = self.offset.clamp(0.0, self.max_offset.max(0.0));
        }

        self.area_id = ui.simple_area_manager.push(
            SimpleArea::new(*bbox, depth).activate_scroll());
    }

    fn send(&mut self, context: &mut (&mut SimpleShapeRenderer<'a>, &mut ScissorStack<'a>)) {
        let (simple_shape_renderer, groups) = context;
        simple_shape_renderer.send_render_box(
            &self.bbox,
            self.bgcolor,
            self.depth,
            None,
            groups,
        );
    }
}
