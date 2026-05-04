use crate::{graphics::renderer::positioning::{PositionMode, RenderBox}, ui::{HasUi, events::ActionType, simple_area::SimpleArea, simple_area_manager::AreaId, ui_element::UiElement}};
use nalgebra_glm as glm;
use winit::window::CursorIcon;

#[derive(Debug)]
struct GrabbingState {
    grabbed: u32,
    offset: f32,
}

impl Default for GrabbingState {
    fn default() -> Self {
        Self { grabbed: 0xffffffff, offset: 0.0 }
    }
}

impl GrabbingState {
    fn grab(&mut self, idx: u32) {
        self.grabbed = idx;
        self.offset = 0.0;
    }

    fn release(&mut self) {
        self.grabbed = 0xffffffff;
    }

    fn on_move(&mut self, delta: f32) {
        self.offset += delta;
    }

    fn get_offset(&self) -> Option<f32> {
        if self.grabbed != 0xffffffff {
            Some(self.offset)
        } else {
            None
        }
    }

    fn get_grabbed(&self) -> Option<u32> {
        if self.grabbed != 0xffffffff {
            Some(self.grabbed)
        } else {
            None
        }
    }

    fn is_grabbing(&self) -> bool {
        self.grabbed != 0xffffffff
    }
}



pub trait HasId {
    fn update_id(&mut self, id: AreaId);
    fn get_id(&self) -> AreaId;
}

#[derive(Debug, Default)]
pub struct Dnd<T> {
    elements: Vec<T>,
    element_size: glm::Vec2,
    gap: f32,
    grab: GrabbingState,
}

impl<T> Dnd<T> {
    pub fn new(element_size: glm::Vec2) -> Self {
        Self {
            elements: vec![],
            element_size,
            gap: 0.0,
            grab: Default::default(),
        }
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn push(&mut self, element: T) {
        self.elements.push(element);
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn get_elements(&self) -> &[T] {
        &self.elements
    }

    pub fn remove(&mut self, idx: usize) -> Option<T> {
        if idx < self.elements.len() {
            Some(self.elements.remove(idx))
        } else {
            None
        }
    }
}

impl<T: HasId + UiElement<UC, SC, GOC>, UC: HasUi, SC, GOC> UiElement<UC, SC, GOC> for Dnd<T> {
    fn update(&mut self, bbox: &RenderBox, depth: u16, context: &mut UC) {
        let height_per_element = self.element_size.y + self.gap;
        let ui = context.get_ui();

        if
            let ActionType::Grabbing(id, _) = ui.events.mouse_actions &&
            !self.grab.is_grabbing() &&
            let Some(index) = self.elements.iter().position(|e| e.get_id() == id)
        {
            self.grab.grab(index as u32);
        }

        let mut target_index = self.grab.get_grabbed().unwrap_or(0) as usize;
        
        if let Some(grabbed_idx) = self.grab.get_grabbed() {
            let delta = ui.ndc_mouse_position.y - ui.previous_ndc_mouse_position.y;
            self.grab.on_move(delta);

            let current_offset = self.grab.get_offset().unwrap_or(0.0);
            let current_y_relative = -(grabbed_idx as f32 * height_per_element) + current_offset;
            let hover_idx = (-current_y_relative / height_per_element).round() as isize;
            target_index = hover_idx.clamp(0, (self.elements.len() - 1) as isize) as usize;

            if matches!(ui.events.mouse_actions, ActionType::AreaRelease(_, _) | ActionType::NoneRelease) {
                let from = grabbed_idx as usize;
                let to = target_index as usize;

                if from != to {
                    let item = self.elements.remove(from);
                    self.elements.insert(to, item);
                }
                
                self.grab.release();
            }
        }

        let is_grabbing = self.grab.is_grabbing();
        let grabbed_idx = self.grab.get_grabbed().unwrap_or(0) as usize;
        let current_offset = self.grab.get_offset().unwrap_or(0.0);
        let total_elements = self.elements.len() as f32;

        for (i, element) in self.elements.iter_mut().enumerate() {
            let mut visual_offset = 0.0;

            if is_grabbing {
                if i == grabbed_idx {
                    let max_up = i as f32 * height_per_element;
                    let max_down = (i as f32 - (total_elements - 1.0)) * height_per_element;
            
                    visual_offset = current_offset.clamp(max_down, max_up);
                } else {
                    if i > grabbed_idx && i <= target_index {
                        visual_offset = height_per_element; 
                    } else if i < grabbed_idx && i >= target_index {
                        visual_offset = -height_per_element;
                    }
                }
            }

            let element_bbox = RenderBox::new_simple(
                bbox.get_top_left() + glm::vec2(0.0, -height_per_element * (i as f32) + visual_offset),
                PositionMode::TopLeft,
                self.element_size,
            );

            let is_this_grabbed = is_grabbing && i == grabbed_idx;
            let effective_depth = if is_this_grabbed { depth.saturating_sub(50) } else { depth };

            element.update_id(context.get_ui().simple_area_manager.push(
                SimpleArea::new(element_bbox, effective_depth).with_cursor(CursorIcon::Grab)
            ));
            
            element.update(&element_bbox, effective_depth - 1, context);
        }
    }

    fn send(&mut self, context: &mut SC) {
        for entry in self.elements.iter_mut() {
            entry.send(context);
        }
    }

    fn generate_output(&mut self, context: &mut GOC) {
        for entry in self.elements.iter_mut() {
            entry.generate_output(context);
        }
    }
}

