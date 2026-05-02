use nalgebra_glm as glm;
use crate::{
    game::{render::UI_BASE_DEPTH, scene::{EventReceiver, EventType, UiElement}},
    graphics::renderer::positioning::{PositionMode, RenderBox},
    ui::{AreaManager, Ui, simple_area::SimpleArea}
};

pub trait HasId {
    fn get_id(&self) -> usize;
}

pub trait HasUi {
    fn get_ui(&mut self) -> &mut Ui;
}

#[derive(Debug)]
pub struct Dnd<T> {
    pub top_left_position: glm::Vec2,
    pub unit_size: glm::Vec2,
    pub gap: f32,
    pub depth: u16,
    elements: Vec<T>,
    grabbed: Option<(usize, f32)>,
}

impl<T: HasId> Dnd<T> {
    pub fn new<F>(
        top_left_position: glm::Vec2,
        unit_size: glm::Vec2,
        gap: f32,
        depth: u16,
        amount: usize,
        ui: &mut Ui,
        constructor: F,
    ) -> Self
    where
        F: Fn(usize, &RenderBox, u16, usize, &mut Ui) -> T,
    {
        let elements = (0..amount)
            .map(|i| {
                let bbox = Self::get_index_bounding_box(top_left_position, unit_size, gap, i);
                let drag_id = ui.drag_area_manager.push(SimpleArea::new(bbox, depth));
                constructor(i, &bbox, depth + 1, drag_id, ui)
            })
            .collect();

        Self {
            top_left_position,
            unit_size,
            gap,
            depth,
            elements,
            grabbed: None,
        }
    }

    pub fn push<F>(&mut self, ui: &mut Ui, constructor: F)
    where
        F: FnOnce(usize, &RenderBox, u16, usize, &mut Ui) -> T,
    {
        let new_idx = self.elements.len();
        let bbox = Self::get_index_bounding_box(self.top_left_position, self.unit_size, self.gap, new_idx);
        
        let drag_id = ui.drag_area_manager.push(SimpleArea::new(bbox, UI_BASE_DEPTH - 1));
        let element = constructor(new_idx, &bbox, self.depth + 1, drag_id, ui);
        
        self.elements.push(element);
    }

    /// Remove um item pela sua posição visual (índice no vetor).
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.elements.len() { return None; }
        
        let element = self.elements.remove(index);
        
        // Se o item removido era o que estava sendo arrastado, cancela o drag
        if let Some((grab_id, _)) = self.grabbed {
            if grab_id == element.get_id() {
                self.grabbed = None;
            }
        }
        
        Some(element)
    }

    fn get_index_bounding_box(top_left: glm::Vec2, unit_size: glm::Vec2, gap: f32, index: usize) -> RenderBox {
        RenderBox::new_simple(
            top_left - glm::vec2(0.0, index as f32 * (unit_size.y + gap)),
            PositionMode::TopLeft,
            unit_size
        )
    }

    fn calculate_target_index(&self, current_idx: usize, offset: f32) -> usize {
        let space = self.unit_size.y + self.gap;
        let shift = (-offset / space).round() as isize;
        (current_idx as isize + shift).clamp(0, self.elements.len() as isize - 1) as usize
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.elements.iter()
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }
}

impl<T: EventReceiver<EC> + HasId, EC> EventReceiver<EC> for Dnd<T> {
    fn propagate_children(&mut self, event: &EventType, context: &mut EC) {
        for element in self.elements.iter_mut() {
            element.on_event(event, context);
        }
    }

    fn on_released(&mut self, id: usize, _context: &mut EC) {
        let Some((grab_id, offset)) = self.grabbed.take() else { return };
        if grab_id != id { return }

        if let Some(old_idx) = self.elements.iter().position(|el| el.get_id() == id) {
            let new_idx = self.calculate_target_index(old_idx, offset);
            if old_idx != new_idx {
                let item = self.elements.remove(old_idx);
                self.elements.insert(new_idx, item);
            }
        }
    }

    fn on_moved(&mut self, id: usize, movement: glm::Vec2, _context: &mut EC) {
        if let Some((area_id, offset)) = &mut self.grabbed {
            if *area_id != id {
                *area_id = id;
                *offset = movement.y;
            } else {
                *offset += movement.y;
            }
        } else {
            self.grabbed = Some((id, movement.y));
        }
    }
}

impl<T: UiElement<UC, SC, GOC> + HasId, UC: HasUi, SC, GOC> UiElement<UC, SC, GOC> for Dnd<T> {
    fn update(&mut self, bbox: &RenderBox, depth: u16, context: &mut UC) {
        self.top_left_position = bbox.get_top_left();
        let space = self.unit_size.y + self.gap;
        
        let drag_info = self.grabbed.and_then(|(id, offset)| {
            let orig_idx = self.elements.iter().position(|el| el.get_id() == id)?;
            let target_idx = self.calculate_target_index(orig_idx, offset);
            Some((orig_idx, target_idx, id, offset))
        });

        for (i, element) in self.elements.iter_mut().enumerate() {
            let static_box = Self::get_index_bounding_box(self.top_left_position, self.unit_size, self.gap, i);
            let element_id = element.get_id();

            let (visual_box, is_grabbed) = if let Some((orig, target, g_id, offset)) = drag_info {
                if element_id == g_id {
                    (static_box.apply_offset(glm::vec2(0.0, offset)), true)
                } else {
                    let shift = if orig < target && i > orig && i <= target {
                        space
                    } else if orig > target && i < orig && i >= target {
                        -space
                    } else {
                        0.0
                    };
                    (static_box.apply_offset(glm::vec2(0.0, shift)), false)
                }
            } else {
                (static_box, false)
            };

            context.get_ui().drag_area_manager.update(
                element_id,
                SimpleArea::new(visual_box.clone(), if is_grabbed { depth - 20 } else { depth }),
            );

            let render_depth = if is_grabbed { depth - 20 } else { depth };
            element.update(&visual_box, render_depth, context);
        }
    }

    fn send(&mut self, context: &mut SC) {
        for element in self.elements.iter_mut() {
            element.send(context);
        }
    }

    fn generate_output(&mut self, context: &mut GOC) {
        for element in self.elements.iter_mut() {
            element.generate_output(context);
        }
    }
}
