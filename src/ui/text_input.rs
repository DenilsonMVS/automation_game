use std::fmt::Debug;

use crate::{graphics::renderer::{fonts::{TextAlign, TextRenderConfig}, positioning::{PositionMode, RenderBox}}, ui::{KeyboardInput, Ui, events::ActionType, rendered_button::RenderedButtonContext, simple_area::SimpleArea, simple_area_manager::AreaId, ui_element::UiElement}};
use nalgebra_glm as glm;
use winit::window::CursorIcon;

const BAR_STATUS_SWITCH: f32 = 0.5;
const CURSOR_PROPORTIONAL_WIDTH: f32 = 0.2;

#[derive(Debug, Default)]
struct BarStatus {
    display: bool,
    time_remaining: f32,
}

impl BarStatus {
    fn on_time_passed(&mut self, amount: f32) {
        self.time_remaining -= amount;
        if self.time_remaining <= 0.0 {
            self.time_remaining += BAR_STATUS_SWITCH;
            self.display = !self.display;
        }
    }
}

pub trait CharFilter {
    fn filter(&self, c: char) -> bool;
}

pub trait ValidateInput {
    fn validate(&self, input: &str) -> bool;
}

#[derive(Debug, Default)]
pub struct TextInput<T: Default + Debug> {
    content: String,
    bbox: RenderBox,
    depth: u16,
    pub bgcolor: glm::U8Vec4,
    hover_bgcolor: glm::U8Vec4,
    focus_bgcolor: glm::U8Vec4,
    color: glm::U8Vec4,
    padding: f32,
    hovered: bool,
    focused: Option<BarStatus>,
    max_len: u32,
    area_id: AreaId,
    text_handling: T,
}

impl<T: Default + Debug> TextInput<T> {
    pub fn new(bgcolor: glm::U8Vec4, color: glm::U8Vec4, text_handling: T) -> Self {
        Self {
            bgcolor,
            hover_bgcolor: bgcolor,
            focus_bgcolor: bgcolor,
            color,
            max_len: u32::MAX,
            text_handling,
            ..Default::default()
        }
    }

    pub fn with_hover_bgcolor(mut self, hover_bgcolor: glm::U8Vec4) -> Self {
        self.hover_bgcolor = hover_bgcolor;
        self
    }

    pub fn with_focus_bgcolor(mut self, focus_bgcolor: glm::U8Vec4) -> Self {
        self.focus_bgcolor = focus_bgcolor;
        self
    }

    pub fn with_max_len(mut self, max_len: u32) -> Self {
        self.max_len = max_len;
        self
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_content(mut self, content: String) -> Self {
        self.content = content;
        self
    }
}

impl <T: Default + Debug + CharFilter + ValidateInput> TextInput<T> {
    pub fn get_content(&self) -> Option<&str> {
        if self.text_handling.validate(&self.content) {
            Some(&self.content)
        } else {
            None
        }
    }
}

impl<'a, 'b, T: Default + Debug + CharFilter + ValidateInput> UiElement<Ui, RenderedButtonContext<'a, 'b>, ()> for TextInput<T> {
    fn update(&mut self, bbox: &RenderBox, depth: u16, ui: &mut Ui) {
        self.bbox = *bbox;
        self.depth = depth;

        self.hovered = matches!(ui.events.mouse_actions, ActionType::AreaHover(id, _) if id == self.area_id);
    
        if let Some(focused) = &mut self.focused {
            let delta = ui.get_delta();
            focused.on_time_passed(delta);
        }

        if ui.events.typing == self.area_id {
            if self.focused.is_none() {
                self.focused = Some(Default::default());
            }
            
            if let KeyboardInput::Text(input) = &ui.keyboard_input {
                for c in input.chars() {
                    if c == '\x08' {
                        self.content.pop();
                    } else if self.text_handling.filter(c) && self.content.len() < self.max_len as usize {
                        self.content.push(c);
                    }
                }
            }
        } else {
            self.focused = None;
        }

        self.area_id = ui.simple_area_manager.push(
            SimpleArea::new(self.bbox, depth).with_cursor(CursorIcon::Text));
    }

    fn send(&mut self, context: &mut RenderedButtonContext) {
        let RenderedButtonContext {
            simple_shape_renderer,
            groups,
            font,
        } = context;

        let char_width = font.get_monospace_width(self.bbox.get_size().y - self.padding * 2.0);
        let content_bbox = self.bbox.cut(self.padding);

        let color = if self.text_handling.validate(&self.content) {
            self.color
        } else {
            glm::vec4(200, 20, 20, 255)
        };
        font.add_text(&TextRenderConfig::new(
            &[(&self.content, color)],
            content_bbox.get_top_left(),
            content_bbox.get_size().y
        ).set_depth(self.depth - 1).set_align(TextAlign::Left).set_mode(PositionMode::TopLeft), groups);

        if let Some(focused) = &self.focused && focused.display {
            let bar_width = char_width * CURSOR_PROPORTIONAL_WIDTH;
            let content_width = self.content.len() as f32 * char_width;
            simple_shape_renderer.send_render_box(
                &RenderBox::new_simple(
                    content_bbox.get_top_left() + glm::vec2(content_width, 0.0),
                    PositionMode::TopLeft,
                    glm::vec2(bar_width, content_bbox.get_size().y),
                ), self.color, self.depth - 1, None, groups,
            );
        }

        let bgcolor = if self.focused.is_some() {
            self.focus_bgcolor
        } else if self.hovered {
            self.hover_bgcolor
        } else {
            self.bgcolor
        };

        simple_shape_renderer.send_render_box(
            &self.bbox,
            bgcolor,
            self.depth,
            None,
            groups,
        );
    }
}
