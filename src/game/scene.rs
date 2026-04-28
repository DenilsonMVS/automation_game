use winit::event::WindowEvent;
use nalgebra_glm as glm;
use crate::MainContext;

#[derive(Debug)]
pub enum EventType {
    Winit(Box<WindowEvent>),
    SimpleAreaPressed(usize),
    Grabbed(usize),
    Moved(Box<(usize, glm::Vec2)>),
    Released(usize),
    FinalEvent,
    HoverDragArea(usize),
    HoverSimpleArea(usize),
}

pub trait Scene {
    fn handle_event(&mut self, event: &EventType, context: &mut crate::MainContext);
    fn game_logic(&mut self, context: &mut MainContext);
    fn send(&mut self, context: &mut MainContext);
    fn generate_output(&mut self, context: &mut MainContext);
}
