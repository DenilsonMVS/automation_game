use crate::{MainContext, graphics::renderer::positioning::RenderBox};

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
