
use nalgebra_glm as glm;

use crate::{MainContext, graphics::renderer::positioning::RenderBox, ui::{Ui, events::ActionType, menu::MenuContext, simple_area::SimpleArea, simple_area_manager::AreaId, ui_element::UiElement}};

#[derive(Debug)]
pub struct ColoredTextView {
    area_id: AreaId,
    content: String,
    color: glm::U8Vec4,
    bbox: RenderBox,
    depth: u16,
    bgcolor: glm::U8Vec4,
    hover_bgcolor: glm::U8Vec4,
    hovered: bool,
}

impl ColoredTextView {
    pub fn new(content: String, color: glm::U8Vec4, bgcolor: glm::U8Vec4) -> Self {
        Self {
            content,
            color,
            bbox: Default::default(),
            depth: 0,
            bgcolor,
            hover_bgcolor: bgcolor.map(|x| x.saturating_add(15)),
            hovered: false,
            area_id: Default::default(),
        }
    }

    pub fn get_len(&self) -> usize {
        self.content.len()
    }
}

impl<'a, 'b, 'c> UiElement<Ui, MainContext<'a, 'b>, ()> for ColoredTextView {
    fn update(&mut self, bbox: &RenderBox, depth: u16, ui: &mut Ui) {
        self.bbox = *bbox;
        self.depth = depth;

        self.hovered = matches!(ui.events.mouse_actions, ActionType::AreaHover(id, _) if id == self.area_id);

        self.area_id = ui.simple_area_manager.push(SimpleArea::new(*bbox, depth));
    }

    fn send(&mut self, context: &mut MainContext) {
        let color = if self.hovered { self.hover_bgcolor } else { self.bgcolor };
        context.simple_shape_renderer.send_render_box(&self.bbox, color, self.depth, 1);
    }
}

#[derive(Debug)]
pub struct ColoredTextButton {
    view: ColoredTextView,
}

impl ColoredTextButton {
    pub fn new(content: String, color: glm::U8Vec4, bgcolor: glm::U8Vec4) -> Self {
        Self { view: ColoredTextView::new(content, color, bgcolor) }
    }
}

impl<'a, 'b, 'c> UiElement<MenuContext<'c, Ui, ColoredTextView>, MainContext<'a, 'b>, ()> for ColoredTextButton {
    fn update(&mut self, bbox: &RenderBox, depth: u16, context: &mut MenuContext<'c, Ui, ColoredTextView>) {
        self.view.update(bbox, depth, context.base_context);
    }

    fn send(&mut self, context: &mut MainContext) {
        self.view.send(context);
    }
}
