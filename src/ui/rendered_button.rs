use nalgebra_glm as glm;
use winit::window::CursorIcon;
use crate::{graphics::renderer::{fonts::{Font, TextAlign, TextRenderConfig}, positioning::{PositionMode, RenderBox}, scissor_group::ScissorStack, simple_shape_renderer::SimpleShapeRenderer}, ui::{Ui, events::ActionType, simple_area::SimpleArea, simple_area_manager::AreaId, ui_element::UiElement}};

#[derive(Debug, Default)]
pub struct RenderedButton {
    pub id: AreaId,
    depth: u16,
    hovered: bool,
    pub disabled: bool,
    bbox: RenderBox,
    color: glm::U8Vec4,
    hover_color: glm::U8Vec4,
    bgcolor: glm::U8Vec4,
    hover_bgcolor: glm::U8Vec4,
    disabled_bgcolor: glm::U8Vec4,
    content: &'static str,
    padding: f32,
}

impl RenderedButton {
    pub fn new(bgcolor: glm::U8Vec4) -> Self {
        Self {
            bgcolor,
            hover_bgcolor: bgcolor.map(|x| x.saturating_add(20)),
            disabled_bgcolor: glm::vec4(120, 120,120, 255),
            ..Default::default()
        }
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_content(mut self, content: &'static str, color: glm::U8Vec4) -> Self {
        self.content = content;
        self.color = color;
        self.hover_color = color;
        self
    }

    pub fn with_hover_bgcolor(mut self, hover_bgcolor: glm::U8Vec4) -> Self {
        self.hover_bgcolor = hover_bgcolor;
        self
    }

    pub fn with_hover_color(mut self, hover_color: glm::U8Vec4) -> Self {
        self.hover_color = hover_color;
        self
    }

    pub fn with_disabled_bgcolor(mut self, disabled_bgcolor: glm::U8Vec4) -> Self {
        self.disabled_bgcolor = disabled_bgcolor;
        self
    }
}

pub struct RenderedButtonContext<'a, 'b> {
    pub font: &'a mut Font<'b>,
    pub simple_shape_renderer: &'a mut SimpleShapeRenderer<'b>,
    pub groups: &'a mut ScissorStack<'b>,
}

impl<'a, 'b> UiElement<Ui, RenderedButtonContext<'a, 'b>, ()> for RenderedButton {
    fn update(&mut self, bbox: &RenderBox, depth: u16, ui: &mut Ui) {
        self.hovered = matches!(ui.events.mouse_actions, ActionType::AreaHover(id, _) if id == self.id);
        self.bbox = *bbox;
        self.depth = depth;

        self.id = ui.simple_area_manager.push(
            SimpleArea::new(*bbox, depth)
                .with_cursor(if self.disabled { CursorIcon::Default } else { CursorIcon::Pointer }));
    }

    fn send(&mut self, context: &mut RenderedButtonContext<'a, 'b>) {
        let RenderedButtonContext { groups, font, simple_shape_renderer} = context;
        let (color, bgcolor) = if self.hovered {
            (self.hover_color, self.hover_bgcolor)
        } else {
            (self.color, self.bgcolor)
        };

        let bgcolor = if self.disabled {
            self.disabled_bgcolor
        } else {
            bgcolor
        };

        if bgcolor.w != 0 {
            simple_shape_renderer.send_render_box(&self.bbox, bgcolor, self.depth, None, groups);
        }

        if color.w != 0 {
            let content_box = self.bbox.cut(self.padding);
            font.add_text(
                &TextRenderConfig::new(&
                    [(self.content, color)],
                    content_box.get_top_left(), content_box.get_size().y,
                )
                .set_mode(PositionMode::TopLeft)
                .set_align(TextAlign::Center)
                .set_depth(self.depth - 1)
                .set_max_line_width(content_box.get_size().x),
                context.groups,
            );
        }
    }
}
