use crate::{graphics::renderer::{positioning::{PositionMode, RenderBox}, simple_shape_renderer::HasSimpleShapeRenderer}, ui::{HasUi, events::ActionType, scroll_area::ScrollArea, simple_area::SimpleArea, simple_area_manager::AreaId, ui_element::UiElement}};
use nalgebra_glm as glm;
use winit::{window::CursorIcon};

pub trait HasKey {
    type Key;
    fn get_key(&self) -> Self::Key;
}

// T and D MUST BE DIFFERENT
#[derive(Debug)]
pub struct Menu<T, D> {
    open_button_id: AreaId,
    items: Vec<T>,
    displayed_value: Option<D>,
    bbox: RenderBox,
    depth: u16,
    opened: bool,
    hovered: bool,
    gap: f32,
    opened_container: ScrollArea,
    maximum_displayed: usize,
}

impl<T, D> Menu<T, D> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            open_button_id: AreaId::default(),
            maximum_displayed: items.len(),
            items,
            bbox: Default::default(),
            depth: Default::default(),
            gap: Default::default(),
            opened_container: ScrollArea::new(glm::vec4(0, 0, 0, 0)),
            opened: false,
            displayed_value: None,
            hovered: false,
        }
    }

    fn get_outer_bounding_box(open_bbox: &RenderBox, gap: f32, amount: usize) -> RenderBox {
        let unit_size = open_bbox.get_size();
        RenderBox::new_simple(
            open_bbox.get_top_right() + glm::vec2(0.0, gap),
            PositionMode::TopLeft,
            glm::vec2(
                unit_size.x + gap * 2.0,
                (unit_size.y + gap) * amount as f32 + gap,
            ),
        )
    }

    fn get_item_bounding_box(open_bbox: &RenderBox, gap: f32, index: usize) -> RenderBox {
        let unit_size = open_bbox.get_size();
        RenderBox::new_simple(
            open_bbox.get_top_right() + glm::vec2(gap, -(unit_size.y + gap) * index as f32),
            PositionMode::TopLeft,
            unit_size,
        )
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn with_open_bgcolor(mut self, color: glm::U8Vec4) -> Self {
        self.opened_container.bgcolor = color;
        self
    }

    pub fn with_maximum_displayed(mut self, amount: usize) -> Self {
        self.maximum_displayed = amount;
        self
    }

    pub fn with_scroll_bar(mut self, width: f32, color: glm::U8Vec4) -> Self {
        self.opened_container = self.opened_container.with_scroll_bar(width, color);
        self
    }
}

impl<T: HasKey, D: HasKey> Menu<T, D> {
    pub fn get_selected_key(&self) -> Option<D::Key> {
        self.displayed_value.as_ref().map(|v| v.get_key())
    }
}

pub struct MenuContext<'a, C, D> {
    pub base_context: &'a mut C,
    pub displayed_value: &'a mut Option<D>,
    pub opened: &'a mut bool,
}

impl<'a, T, D, UC, SC, GOC> UiElement<UC, SC, GOC> for Menu<T, D>
where
    UC: HasUi,
    SC: HasSimpleShapeRenderer<'a>,
    // D é puramente visual (ex: ColoredTextView)
    D: UiElement<UC, SC, GOC>, 
    // T é interativo e "conversa" com o menu (ex: ColoredTextButton)
    for<'b> T: UiElement<MenuContext<'b, UC, D>, SC, GOC>,
{
    fn update(&mut self, bbox: &RenderBox, depth: u16, context: &mut UC) {
        self.bbox = *bbox;
        self.depth = depth;

        let mouse_actions = context.get_ui().events.mouse_actions;
        match mouse_actions {
            ActionType::AreaPress(id, depth) => {
                if id == self.open_button_id {
                    self.opened = true;
                } else if depth > self.depth {
                    self.opened = false;
                }
            },
            ActionType::NonePress => {
                self.opened = false;
            },
            _ => {},
        };
        if matches!(mouse_actions, ActionType::AreaPress(id, _) if id == self.open_button_id) {
            self.opened = true;
        } else if mouse_actions == ActionType::NonePress {
            self.opened = false;
        }

        self.open_button_id = context.get_ui().simple_area_manager.push(SimpleArea::new(*bbox, depth - 20).with_cursor(CursorIcon::Pointer));
        self.hovered = matches!(mouse_actions, ActionType::AreaHover(id, _) if id == self.open_button_id);

        if self.opened {
            let outer_box = Self::get_outer_bounding_box(&self.bbox, self.gap, self.maximum_displayed);
            let total_displayed_box = Self::get_outer_bounding_box(&self.bbox, self.gap, self.items.len());
            self.opened_container.max_offset = total_displayed_box.get_size().y - outer_box.get_size().y;
            self.opened_container.update(&outer_box, depth - 100, context.get_ui());

            for (i, item) in self.items.iter_mut().enumerate() {
                let item_bbox = Self::get_item_bounding_box(&self.bbox, self.gap, i);
                let item_bbox = self.opened_container.apply_offset_to_box(&item_bbox);
                let mut item_context = MenuContext {
                    base_context: context,
                    displayed_value: &mut self.displayed_value,
                    opened: &mut self.opened,
                };
                item.update(&item_bbox, depth - 101, &mut item_context);
            }
        }

        if let Some(displayed_value) = &mut self.displayed_value {
            displayed_value.update(bbox, depth - 1, context);
        }
    }

    fn send(&mut self, context: &mut SC) {
        if self.opened {
            if self.opened_container.max_offset > 0.0 {
                self.opened_container.push_scissor_context(context.get_groups());
            }

            for item in self.items.iter_mut() {
                item.send(context);
            }

            if self.opened_container.max_offset > 0.0 {
                context.get_groups().pop();   
            }
            
            self.opened_container.send(&mut context.get());
        }

        if let Some(displayed_value) = &mut self.displayed_value {
            displayed_value.send(context);
        }

        let (simple_shape_renderer, groups) = context.get();
        if self.hovered {
            simple_shape_renderer.send_render_box(&self.bbox, glm::vec4(255, 255, 255, 40), self.depth - 20, None, groups);
        } else if self.displayed_value.is_none() {
            simple_shape_renderer.send_render_box(&self.bbox, glm::vec4(255, 255, 255, 20), self.depth - 20, None, groups);
        }
    }

    fn generate_output(&mut self, context: &mut GOC) {
        if self.opened {
            for item in self.items.iter_mut() {
                item.generate_output(context);
            }
        }

        if let Some(displayed_value) = &mut self.displayed_value {
            displayed_value.generate_output(context);
        }
    }
}
