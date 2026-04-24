use crate::{graphics::{camera::Camera, renderer::positioning::RenderBox}, ui::{Ui, UiContext}};
use nalgebra_glm as glm;
use winit::window::{Cursor, CursorIcon};

pub struct ButtonArea {
    bounding_box: RenderBox,
    on_click: Box<dyn Fn(&mut UiContext, &Ui)>,
}

impl ButtonArea {
    pub fn new<T>(bounding_box: RenderBox, on_click: T) -> Self
    where T: Fn(&mut UiContext, &Ui) + 'static
    {
        Self { bounding_box, on_click: Box::new(on_click) }
    }

    pub fn new_from_world_pos<T>(world_box: RenderBox, on_click: T, camera: &Camera) -> Self
    where T: Fn(&mut UiContext, &Ui) + 'static
    {
        Self {
            bounding_box: RenderBox {
                min: camera.world_to_ndc_pos(world_box.min),
                max: camera.world_to_ndc_pos(world_box.max),
            },
            on_click: Box::new(on_click),
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