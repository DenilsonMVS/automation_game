use crate::{MainContext, game::{render::{UI_BASE_DEPTH, robot_info::{get_background_color, get_hovered_color, get_secondary_background_color}}, scene::EventReceiver}, graphics::renderer::{fonts::{TextAlign, TextRenderConfig}, positioning::{PositionMode, RenderBox}}, ui::{AreaManager, Ui, simple_area::SimpleArea}};
use nalgebra_glm as glm;

const MENU_BASE_DEPTH: u16 = UI_BASE_DEPTH - 30;

#[derive(Debug)]
pub struct Menu<T> {
    open_button_bbox: RenderBox,
    entry_size: glm::Vec2,
    open_button_id: usize,
    options: Vec<(T, glm::U8Vec4)>,
    entries_button_ids: Vec<usize>,
    selected_entry: Option<usize>,
    hovered: bool,
    pub padding: f32,
}

impl<T: std::fmt::Display + Clone> Menu<T> {
    pub fn new(open_button_bbox: RenderBox, entry_size: glm::Vec2, options: Vec<(T, glm::U8Vec4)>, ui: &mut Ui) -> Self {
        Self {
            open_button_bbox,
            entry_size,
            open_button_id: ui.simple_area_manager.push(SimpleArea::new_button(
                open_button_bbox,
                MENU_BASE_DEPTH,
            )),
            options,
            entries_button_ids: vec![],
            selected_entry: None,
            hovered: false,
            padding: 0.01,
        }
    }

    pub fn get_selected_value(&self) -> Option<T> {
        self.selected_entry
            .and_then(|idx| self.options.get(idx))
            .map(|(value, _)| value.clone())
    }

    fn get_menu_idx_bbox(&self, i: usize) -> RenderBox {
        RenderBox::new_simple(
            self.open_button_bbox.get_top_right() + glm::vec2(self.padding, -((i as f32) * (self.entry_size.y + self.padding))),
            PositionMode::TopLeft,
            self.entry_size,
        )
    }

    fn get_menu_outer_bbox(&self) -> RenderBox {
        RenderBox::new_simple(
            self.open_button_bbox.get_top_right() + glm::vec2(0.0, self.padding),
            PositionMode::TopLeft,
            glm::vec2(
                self.entry_size.x + self.padding * 2.0,
                (self.entry_size.y + self.padding) * (self.entries_button_ids.len() as f32) + self.padding,
            ),
        )
    }

    pub fn update(&mut self, top_left_position: glm::Vec2, ui: &mut Ui) {
        let diff = top_left_position - self.open_button_bbox.get_top_left();
        self.open_button_bbox = self.open_button_bbox.apply_offset(diff);
        ui.simple_area_manager.update(self.open_button_id, SimpleArea::new_button(
            self.open_button_bbox,
            MENU_BASE_DEPTH,
        ));

        for (i, entry_button_id) in self.entries_button_ids.iter().enumerate() {
            ui.simple_area_manager.update(
                *entry_button_id,
                SimpleArea::new_button(
                    self.get_menu_idx_bbox(i),
                    MENU_BASE_DEPTH - 2,
                ),
            );
        }
    }

    pub fn send(&mut self, context: &mut MainContext) {
        if self.hovered {
            context.simple_shape_renderer.send_render_box(
                &self.open_button_bbox,
                get_hovered_color(),
                UI_BASE_DEPTH - 3,
                1,
            );
            self.hovered = false;
        }

        for i in 0..self.entries_button_ids.len() {
            let bbox = Self::get_menu_idx_bbox(&self, i);
            context.simple_shape_renderer.send_render_box(
                &bbox,
                get_secondary_background_color(),
                MENU_BASE_DEPTH - 1,
                1,
            );

            if let Some((content, color)) = self.options.get(i) {
                let label = content.to_string();
                context.font.add_text(
                    &TextRenderConfig::new(
                        &[(&label, *color)],
                        bbox.get_top_left(),
                        bbox.get_size().y
                    ).set_align(TextAlign::Left).set_mode(PositionMode::TopLeft).set_depth(MENU_BASE_DEPTH - 2),
                );
            }
        }

        if !self.entries_button_ids.is_empty() {
            context.simple_shape_renderer.send_render_box(
                &self.get_menu_outer_bbox(),
                get_background_color(),
                MENU_BASE_DEPTH,
                1,
            );
        }

        if
            let Some(idx) = self.selected_entry &&
            let Some((content, color)) = self.options.get(idx)
        {
            let label = content.to_string();
            context.font.add_text(
                &TextRenderConfig::new(
                    &[(&label, *color)],
                    self.open_button_bbox.get_top_left(),
                    self.open_button_bbox.get_size().y
                ).set_align(TextAlign::Left).set_mode(PositionMode::TopLeft).set_depth(UI_BASE_DEPTH - 4),
            );
        }
    }
}

impl<T: std::fmt::Display + Clone> EventReceiver<Ui> for Menu<T> {
    fn on_hover_simple(&mut self, id: usize, _context: &mut Ui) {
        if id == self.open_button_id {
            self.hovered = true;
        }
    }

    fn on_simple_area_pressed(&mut self, id: usize, ui: &mut Ui) {
        if id == self.open_button_id {
            self.entries_button_ids.clear();
            for i in 0..self.options.len() {
                self.entries_button_ids.push(ui.simple_area_manager.push(SimpleArea::new_button(
                    self.get_menu_idx_bbox(i),
                    MENU_BASE_DEPTH - 2,
                )));
            }
        } else {
            for (i, button_id) in self.entries_button_ids.iter().enumerate() {
                if *button_id == id {
                    self.selected_entry = Some(i);
                    self.entries_button_ids.clear();
                    break;
                }
            }
        }
    }
}