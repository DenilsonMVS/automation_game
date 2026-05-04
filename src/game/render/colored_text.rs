
use nalgebra_glm as glm;
use winit::window::CursorIcon;

use crate::{MainContext, graphics::renderer::{fonts::{TextAlign, TextRenderConfig}, positioning::{PositionMode, RenderBox}}, ui::{HasUi, Ui, events::ActionType, menu::MenuContext, simple_area::SimpleArea, simple_area_manager::AreaId, ui_element::UiElement}};

#[derive(Debug)]
pub struct ColoredTextView<Key> {
    pub area_id: AreaId,
    key: Key,
    color: glm::U8Vec4,
    bbox: RenderBox,
    depth: u16,
    bgcolor: glm::U8Vec4,
    hover_bgcolor: glm::U8Vec4,
    hovered: bool,
}

impl<Key: Clone> ColoredTextView<Key> {
    pub fn new(key: Key, color: glm::U8Vec4, bgcolor: glm::U8Vec4) -> Self {
        Self {
            key,
            color,
            bbox: Default::default(),
            depth: 0,
            bgcolor,
            hover_bgcolor: bgcolor.map(|x| x.saturating_add(15)),
            hovered: false,
            area_id: Default::default(),
        }
    }

    pub fn create_display_clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            color: self.color,
            bbox: Default::default(),
            depth: 0,
            bgcolor: self.bgcolor,
            hover_bgcolor: self.hover_bgcolor,
            hovered: false,
            area_id: Default::default(),
        }
    }
}

impl<'a, 'b, 'c, Key: std::fmt::Display> UiElement<Ui, MainContext<'a, 'b>, ()> for ColoredTextView<Key> {
    fn update(&mut self, bbox: &RenderBox, depth: u16, ui: &mut Ui) {
        self.bbox = *bbox;
        self.depth = depth;

        self.hovered = matches!(ui.events.mouse_actions, ActionType::AreaHover(id, _) if id == self.area_id);

        self.area_id = ui.simple_area_manager.push(SimpleArea::new(*bbox, depth).with_cursor(CursorIcon::Pointer));
    }

    fn send(&mut self, context: &mut MainContext) {
        let bbox_size = self.bbox.get_size();
        let color = if self.hovered { self.hover_bgcolor } else { self.bgcolor };
        context.simple_shape_renderer.send_render_box(&self.bbox, color, self.depth, None);
        let label = format!("{}", self.key);
        context.font.add_text(
            &TextRenderConfig::new(
                &[(&label, self.color)],
                self.bbox.get_top_left(),
                bbox_size.y,
            )
            .set_max_line_width(bbox_size.x)
            .set_align(TextAlign::Left)
            .set_depth(self.depth - 1)
            .set_mode(PositionMode::TopLeft),
        );
    }
}

#[derive(Debug)]
pub struct ColoredTextButton<Key> {
    view: ColoredTextView<Key>,
}

impl<Key: Clone> ColoredTextButton<Key> {
    pub fn new(key: Key, color: glm::U8Vec4, bgcolor: glm::U8Vec4) -> Self {
        Self { view: ColoredTextView::new(key, color, bgcolor) }
    }
}

impl<'a, 'b, 'c, Key: std::fmt::Display + Clone> UiElement<MenuContext<'c, Ui, ColoredTextView<Key>>, MainContext<'a, 'b>, ()> for ColoredTextButton<Key> {
    fn update(&mut self, bbox: &RenderBox, depth: u16, context: &mut MenuContext<'c, Ui, ColoredTextView<Key>>) {
        self.view.update(bbox, depth, context.base_context);

        let ui = context.base_context.get_ui();
        if matches!(ui.events.mouse_actions, ActionType::AreaPress(id, _) if id == self.view.area_id) {
            *context.opened = false;
            *context.displayed_value = Some(self.view.create_display_clone());
        }
    }

    fn send(&mut self, context: &mut MainContext) {
        self.view.send(context);
    }
}
