use winit::window::CursorIcon;
use nalgebra_glm as glm;
use crate::{graphics::renderer::{positioning::RenderBox, simple_shape_renderer::SimpleShapeRenderer}, ui::{Ui, rendered_area::RenderedArea, ui_element::UiElement}};

pub fn get_background_color() -> glm::U8Vec4 {
    glm::vec4(8, 17, 25, 255)
}


#[derive(Debug)]
pub struct RobotInfo {
    pub robot_idx: usize,
    outer_container: RenderedArea,
}

impl RobotInfo {
    pub fn new(robot_idx: usize) -> Self {
        Self { robot_idx, outer_container: RenderedArea::new(CursorIcon::Default, get_background_color()) }
    }
}

impl<'a> UiElement<Ui, SimpleShapeRenderer<'a>, ()> for RobotInfo {
    fn update(&mut self, bbox: &RenderBox, depth: u16, context: &mut Ui) {
        self.outer_container.update(bbox, depth, context);
    }

    fn send(&mut self, context: &mut SimpleShapeRenderer<'a>) {
        self.outer_container.send(context);
    }
}
