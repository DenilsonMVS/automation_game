use std::marker::PhantomData;
use crate::{game::scene::UiElement, graphics::renderer::{positioning::RenderBox, simple_shape_renderer::SimpleShapeRenderer}, ui::{AreaManager, simple_area::SimpleArea}};
use nalgebra_glm as glm;

#[derive(Debug)]
pub struct RenderedArea<T: AreaManager> {
    id: usize,
    color: glm::U8Vec4,
    _manager: PhantomData<T>,
}

impl<T: AreaManager> RenderedArea<T> {
    pub fn new(bbox: RenderBox, depth: u16, color: glm::U8Vec4, manager: &mut T) -> Self {
        Self {
            id: manager.push(SimpleArea::new(bbox, depth)),
            color,
            _manager: PhantomData,
        }
    }

    pub fn get_area<'a>(&self, manager: &'a T) -> Option<&'a SimpleArea> {
        manager.get(self.id)
    }
}

pub struct SendContext<'a, 'b, T: AreaManager> {
    pub manager: &'a T,
    pub simple_shape_renderer: &'a mut SimpleShapeRenderer<'b>,
}

impl<'a, 'b, T: AreaManager> UiElement<T, SendContext<'a, 'b, T>, ()> for RenderedArea<T> {
    fn update(&mut self, bbox: &RenderBox, depth: u16, manager: &mut T) {
        manager.update(self.id, SimpleArea::new(*bbox, depth));            
    }

    fn send(&mut self, context: &mut SendContext<'a, 'b, T>) {
        let manager = &context.manager;
        let simple_shape_renderer = &mut context.simple_shape_renderer;
        if let Some(area) = manager.get(self.id) {
            simple_shape_renderer.send_render_box(
                &area.bounding_box,
                self.color,
                area.depth,
                1,
            );
        }
    }
}
