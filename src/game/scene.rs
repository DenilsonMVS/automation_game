use winit::event::WindowEvent;

use crate::MainContext;

pub struct Event<'a> {
    event_type: EventType<'a>,
    received: bool,
}

impl<'a> Event<'a> {
    pub fn new(event_type: EventType<'a>) -> Self {
        Self { event_type, received: false }
    }

    pub fn get_event(&mut self) -> &EventType<'a> {
        self.received = true;
        &self.event_type
    }
}

pub enum EventType<'a> {
    Winit(&'a WindowEvent),
}

pub trait Scene {
    fn handle_event(&mut self, event: Event);
    fn game_logic(&mut self, context: &mut MainContext);
    fn send(&mut self, context: &mut MainContext);
    fn generate_output(&mut self, context: &mut MainContext);
}
