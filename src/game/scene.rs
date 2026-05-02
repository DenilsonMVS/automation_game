use winit::event::WindowEvent;
use nalgebra_glm as glm;
use crate::{MainContext, graphics::renderer::positioning::RenderBox};

#[derive(Debug)]
pub enum EventType {
    Winit(Box<WindowEvent>),
    SimpleAreaPressed(usize),
    Grabbed(usize),
    Moved(Box<(usize, glm::Vec2)>),
    Released(usize),
    HoverDragArea(usize),
    HoverSimpleArea(usize),
    ClickOutside,
}

pub trait EventReceiver<T> {
    fn on_winit(&mut self, _event: &WindowEvent, _context: &mut T) {}
    fn on_simple_area_pressed(&mut self, _id: usize, _context: &mut T) {}
    fn on_grabbed(&mut self, _id: usize, _context: &mut T) {}
    fn on_moved(&mut self, _id: usize, _movement: glm::Vec2, _context: &mut T) {}
    fn on_released(&mut self, _id: usize, _context: &mut T) {}
    fn on_hover_drag(&mut self, _id: usize, _context: &mut T) {}
    fn on_hover_simple(&mut self, _id: usize, _context: &mut T) {}
    fn on_click_outside(&mut self, _context: &mut T) {}

    fn on_event(&mut self, event: &EventType, context: &mut T) {
        match event {
            EventType::Winit(ev) => self.on_winit(ev, context),
            EventType::SimpleAreaPressed(id) => self.on_simple_area_pressed(*id, context),
            EventType::Grabbed(id) => self.on_grabbed(*id, context),
            EventType::Moved(data) => {
                let (id, movement) = **data; 
                self.on_moved(id, movement, context);
            },
            EventType::Released(id) => self.on_released(*id, context),
            EventType::HoverDragArea(id) => self.on_hover_drag(*id, context),
            EventType::HoverSimpleArea(id) => self.on_hover_simple(*id, context),
            EventType::ClickOutside => self.on_click_outside(context),
        };

        self.propagate_children(event, context);
    }

    fn propagate_children(&mut self, _event: &EventType, _context: &mut T) {}
}

pub trait UiElement<UpdateContext, SendContext, GenerateOutputContext> {
    fn update(&mut self, _bbox: &RenderBox, _depth: u16, _context: &mut UpdateContext) {}
    fn send(&mut self, _context: &mut SendContext) {}
    fn generate_output(&mut self, _context: &mut GenerateOutputContext) {}
}

pub trait Scene {
    fn game_logic(&mut self, context: &mut MainContext);
    fn send(&mut self, context: &mut MainContext);
    fn generate_output(&mut self, context: &mut MainContext);
}
