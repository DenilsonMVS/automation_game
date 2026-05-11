use winit::window::CursorIcon;
use nalgebra_glm as glm;
use crate::{graphics::renderer::{positioning::RenderBox, scissor_group::ScissorStack, simple_shape_renderer::SimpleShapeRenderer}, ui::{Ui, events::ActionType, simple_area::SimpleArea, simple_area_manager::AreaId, ui_element::UiElement}};


#[derive(Debug, Default)]
pub struct RenderedArea {
    pub id: AreaId,
    cursor: CursorIcon,
    pub depth: u16,
    hovered: bool,
    pub bbox: RenderBox,
    pub color: glm::U8Vec4,
    pub hover_color: glm::U8Vec4,
}

impl RenderedArea {
    pub fn new(cursor: CursorIcon, color: glm::U8Vec4) -> Self {
        Self { 
            cursor,
            color,
            hover_color: color,
            ..Default::default()
        }
    }

    pub fn with_hover_color(mut self, hover_color: glm::U8Vec4) -> Self {
        self.hover_color = hover_color;
        self
    }
}

impl<'a, 'b> UiElement<Ui, (&'a mut SimpleShapeRenderer<'b>, &'a mut ScissorStack<'b>), ()> for RenderedArea {
    fn update(&mut self, bbox: &RenderBox, depth: u16, ui: &mut Ui) {
        self.bbox = *bbox;
        self.depth = depth;
        self.hovered = matches!(ui.events.mouse_actions, ActionType::AreaHover(id, _) if id == self.id);
        self.id = ui.simple_area_manager.push(SimpleArea::new(self.bbox, self.depth).with_cursor(self.cursor));
    }

    fn send(&mut self, context: &mut (&'a mut SimpleShapeRenderer<'b>, &'a mut ScissorStack)) {
        let (simple_shape_renderer, groups) = context;
        let color = if self.hovered { self.hover_color } else { self.color };
        if color.w != 0 {
            simple_shape_renderer.send_render_box(&self.bbox, color, self.depth, None, groups);
        }
    }
}
