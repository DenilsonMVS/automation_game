use crate::{graphics::renderer::positioning::RenderBox, ui::{Ui, UiContext}};
use nalgebra_glm as glm;

#[derive(Debug)]
pub struct DragArea {
    initial_box: RenderBox,
    offset: glm::Vec2,
    locked_x: bool,
    locked_y: bool,
    on_grab: Option<fn(&Self, &mut UiContext, &Ui)>,
    on_move: Option<fn(&Self, &mut UiContext, &Ui)>,
    on_release: Option<fn(&Self, &mut UiContext, &Ui)>,
}

impl DragArea {
    pub fn new(bounding_box: RenderBox) -> Self {
        Self {
            initial_box: bounding_box,
            offset: glm::vec2(0.0, 0.0),
            on_grab: None, on_move: None, on_release: None,
            locked_x: false, locked_y: false,
        }
    }

    pub fn lock_x(mut self) -> Self {
        self.locked_x = true;
        self
    }

    pub fn lock_y(mut self) -> Self {
        self.locked_y = true;
        self
    }

    pub fn get_current_position(&self) -> RenderBox {
        self.initial_box.apply_offset(self.offset)
    }

    pub fn with_on_grab(mut self, on_grab: fn(&Self, &mut UiContext, &Ui)) -> Self {
        self.on_grab = Some(on_grab);
        self
    }

    pub fn with_on_move(mut self, on_move: fn(&Self, &mut UiContext, &Ui)) -> Self {
        self.on_move = Some(on_move);
        self
    }

    pub fn with_on_release(mut self, on_release: fn(&Self, &mut UiContext, &Ui)) -> Self {
        self.on_release = Some(on_release);
        self
    }

    fn get_lock_multiplier(&self) -> glm::Vec2 {
        glm::vec2(
            if self.locked_x { 0.0 } else { 1.0 },
            if self.locked_y { 0.0 } else { 1.0 },
        )
    }

    pub fn update_offset(&mut self, movement: glm::Vec2) {
        self.offset += movement.component_mul(&self.get_lock_multiplier());
    }

    pub fn on_move(&self, context: &mut UiContext, ui: &Ui) {
        if let Some(on_move) = self.on_move {
            on_move(self, context, ui);
        }
    }

    pub fn on_grab(&self, context: &mut UiContext, ui: &Ui) {
        if let Some(on_grab) = self.on_move {
            on_grab(self, context, ui);
        }
    }

    pub fn on_release(&self, context: &mut UiContext, ui: &Ui) {
        if let Some(on_release) = self.on_release {
            on_release(self, context, ui);
        }
    }
}
