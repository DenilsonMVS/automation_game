use crate::{graphics::{camera::Camera, renderer::positioning::RenderBox}, ui::{Ui, UiContext}};
use nalgebra_glm as glm;
use winit::window::{Cursor, CursorIcon};

#[derive(Debug)]
pub struct ButtonArea {
    bounding_box: RenderBox,
    on_click: fn(&mut UiContext, &Ui),
}

impl ButtonArea {
    pub fn new(bounding_box: RenderBox, on_click: fn(&mut UiContext, &Ui)) -> Self {
        Self { bounding_box, on_click }
    }

    pub fn new_from_world_pos(world_box: RenderBox, on_click: fn(&mut UiContext, &Ui), camera: &Camera) -> Self {
        Self {
            bounding_box: RenderBox {
                min: camera.world_to_ndc_pos(world_box.min),
                max: camera.world_to_ndc_pos(world_box.max),
            },
            on_click,
        }
    }

    pub(super) fn test(&self, click_position: glm::Vec2, context: &mut UiContext, clicked: bool, ui: &Ui) {
        if self.bounding_box.is_point_inside(click_position) {
            context.window.set_cursor(Cursor::Icon(CursorIcon::Pointer));
            if clicked {
                (self.on_click)(context, ui);
            }
        }
    }
}