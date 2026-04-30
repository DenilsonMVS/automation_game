use crate::{MainContext, game::scene::UiElement, graphics::renderer::positioning::RenderBox, ui::{Ui, simple_area::SimpleArea}};
use nalgebra_glm as glm;

#[derive(Debug)]
pub struct RenderedArea {
    id: usize,
    color: glm::U8Vec4,
}

impl RenderedArea {
    pub fn new(bbox: RenderBox, depth: u16, color: glm::U8Vec4, ui: &mut Ui) -> Self {
        Self {
            id: ui.simple_area_manager.push_simple_area(SimpleArea::new(bbox, depth)),
            color,
        }
    }
}

impl<'a, 'b> UiElement<Ui, MainContext<'a, 'b>, ()> for RenderedArea {
    fn update(&mut self, position: glm::Vec2, depth: u16, ui: &mut Ui) {
        let areas = &mut ui.simple_area_manager;
        if let Some(area) = areas.get_area(self.id) {
            let bbox = area.bounding_box;
            let diff = position - bbox.get_top_left();            
            areas.update_simple_area(self.id, SimpleArea::new(bbox.apply_offset(diff), depth));            
        }
    }

    fn send(&mut self, context: &mut MainContext<'a, 'b>) {
        let areas = &mut context.ui.simple_area_manager;
        let simple_shape_renderer = &mut context.simple_shape_renderer;
        if let Some(area) = areas.get_area(self.id) {
            simple_shape_renderer.send_render_box(
                &area.bounding_box,
                self.color,
                area.depth,
                1,
            );
        }
    }
}
