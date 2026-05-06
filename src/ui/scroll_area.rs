use crate::{graphics::renderer::{positioning::{PositionMode, RenderBox}, scissor_group::ScissorStack, simple_shape_renderer::SimpleShapeRenderer}, ui::{Ui, events::ActionType, rendered_area::RenderedArea, simple_area::SimpleArea, simple_area_manager::AreaId, ui_element::UiElement}};
use nalgebra_glm as glm;
use winit::window::CursorIcon;

const SCROLL_SENSITIVITY: f32 = 0.02;

#[derive(Debug, Default)]
pub struct ScrollArea {
    bbox: RenderBox,
    depth: u16,
    pub max_offset: f32,
    offset: f32,
    pub bgcolor: glm::U8Vec4,
    area_id: AreaId,
    scroll_bar_width: f32,
    scroll_area: RenderedArea,
}

impl ScrollArea {
    pub fn new(bgcolor: glm::U8Vec4) -> Self {
        Self {
            bgcolor,
            scroll_area: RenderedArea::new(CursorIcon::Grab, glm::vec4(0, 0, 0, 0)),
            ..Default::default()
        }
    }

    pub fn push_scissor_context(&self, groups: &mut ScissorStack) {
        groups.push(self.bbox);
    }

    pub fn apply_offset_to_box(&self, bbox: &RenderBox) -> RenderBox {
        bbox.apply_offset(glm::vec2(0.0, self.get_offset()))
    }

    pub fn get_bbox(&self) -> &RenderBox {
        &self.bbox
    }

    pub fn get_offset(&self) -> f32 {
        self.offset.clamp(0.0, self.max_offset.max(0.0))
    }

    pub fn with_scroll_bar(mut self, width: f32, color: glm::U8Vec4) -> Self {
        self.scroll_bar_width = width;
        self.scroll_area.color = color;
        self.scroll_area.hover_color = color;
        self
    } 
}

impl<'a, 'b> UiElement<Ui, (&'a mut SimpleShapeRenderer<'b>, &'a mut ScissorStack<'b>), ()> for ScrollArea {
    fn update(&mut self, bbox: &RenderBox, depth: u16, ui: &mut Ui) {
        self.bbox = RenderBox::new_simple(
            bbox.get_top_left(),
            PositionMode::TopLeft,
            bbox.get_size() + glm::vec2(self.scroll_bar_width, 0.0),
        );
        self.depth = depth;

        if ui.events.scroll == self.area_id {
            self.offset -= ui.scroll * SCROLL_SENSITIVITY;
        }

        let container_height = self.bbox.get_size().y;
        let total_height = container_height + self.max_offset;
        let scroll_bar_height_proportion = if total_height > 0.0 { container_height / total_height } else { 1.0 };
        let scroll_bar_height = scroll_bar_height_proportion * container_height;
        let scroll_bar_free_space = container_height - scroll_bar_height;

        if matches!(ui.events.mouse_actions, ActionType::Grabbing(id, _) if id == self.scroll_area.id) {
            let delta_y = ui.ndc_mouse_position.y - ui.previous_ndc_mouse_position.y;    
            if scroll_bar_free_space > 0.0 && self.max_offset > 0.0 {
                let ratio = self.max_offset / scroll_bar_free_space;
                self.offset -= delta_y * ratio;
            }
        }

        let used_offset = self.get_offset();
        let scroll_bar_screen_offset = if self.max_offset > 0.0 {
            (used_offset / self.max_offset) * scroll_bar_free_space
        } else {
            0.0
        };

        self.scroll_area.update(
            &RenderBox::new_simple(
                bbox.get_top_right() - glm::vec2(0.0, scroll_bar_screen_offset),
                PositionMode::TopLeft,
                glm::vec2(self.scroll_bar_width, scroll_bar_height),
            ),
            depth - 1, ui,
        );

        self.area_id = ui.simple_area_manager.push(
            SimpleArea::new(self.bbox, depth).activate_scroll());
    }

    fn send(&mut self, context: &mut (&'a mut SimpleShapeRenderer<'b>, &'a mut ScissorStack<'b>)) {
        self.scroll_area.send(context);
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
