use crate::graphics::renderer::positioning::RenderBox;

pub trait UiElement<UC, SC, GOC> {
    fn update(&mut self, _bbox: &RenderBox, _depth: u16, _context: &mut UC) {}
    fn send(&mut self, _context: &mut SC) {}
    fn generate_output(&mut self, _context: &mut GOC) {}
}
