
use nalgebra_glm as glm;

use crate::{MainContext, game::scene::{EventReceiver, UiElement}, graphics::renderer::{fonts::{TextAlign, TextRenderConfig}, positioning::{PositionMode, RenderBox}}, ui::{AreaManager, Ui, simple_area::SimpleArea}};

#[derive(Debug)]
pub struct RenderedButton {
    id: usize,
    pub bgcolor: glm::U8Vec4,
    pub color: glm::U8Vec4,
    pub hover_bgcolor: glm::U8Vec4,
    pub hover_color: glm::U8Vec4,
    content: &'static str,
    padding: f32,
    hovered: bool,
}

impl RenderedButton {
    pub fn new(bbox: RenderBox, depth: u16, bgcolor: glm::U8Vec4, ui: &mut Ui) -> Self {
        Self {
            id: ui.simple_area_manager.push(SimpleArea::new_button(bbox, depth)),
            bgcolor,
            color: Default::default(),
            content: "",
            padding: 0.0,
            hover_bgcolor: bgcolor,
            hover_color: Default::default(),
            hovered: false,
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn with_color(mut self, color: glm::U8Vec4) -> Self {
        self.color = color;
        self.hover_color = color;
        self
    }

    pub fn with_hover_bgcolor(mut self, color: glm::U8Vec4) -> Self {
        self.hover_bgcolor = color;
        self
    }

    pub fn with_hover_color(mut self, color: glm::U8Vec4) -> Self {
        self.hover_color = color;
        self
    }

    pub fn with_content(mut self, content: &'static str) -> Self {
        self.content = content;
        self
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn get_area<'a>(&self, ui: &'a Ui) -> Option<&'a SimpleArea> {
        ui.simple_area_manager.get(self.id)
    }
}

impl<'a, 'b> UiElement<Ui, MainContext<'a, 'b>, ()> for RenderedButton {
    fn update(&mut self, bbox: &RenderBox, depth: u16, ui: &mut Ui) {       
        ui.simple_area_manager.update(self.id, SimpleArea::new_button(*bbox, depth));
    }

    fn send(&mut self, context: &mut MainContext<'a, 'b>) {
        let areas = &mut context.ui.simple_area_manager;
        let simple_shape_renderer = &mut context.simple_shape_renderer;
        let font = &mut context.font;
        if let Some(area) = areas.get(self.id) {
            let (bgcolor, color) = if self.hovered {
                (self.hover_bgcolor, self.hover_color)
            } else {
                (self.bgcolor, self.color)
            };
            self.hovered = false;

            simple_shape_renderer.send_render_box(
                &area.bounding_box,
                bgcolor,
                area.depth,
                1,
            );

            let padded_area = area.bounding_box.cut(self.padding);
            font.add_text(
                &TextRenderConfig::new(
                    &[(self.content, color)],
                    padded_area.get_top_left(),
                    padded_area.get_size().y,
                )
                .set_max_line_width(padded_area.get_size().x)
                .set_align(TextAlign::Center)
                .set_mode(PositionMode::TopLeft)
                .set_depth(area.depth - 1)
            );
        }
    }
}

impl EventReceiver<()> for RenderedButton {
    fn on_hover_simple(&mut self, id: usize, _context: &mut ()) {
        if id == self.id {
            self.hovered = true;
        }
    }
}
