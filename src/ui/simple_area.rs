use crate::graphics::renderer::positioning::RenderBox;

#[derive(Debug, Clone, Copy)]
pub struct SimpleArea {
    pub bounding_box: RenderBox,
    pub depth: u16,
    pub alive: bool,
    pub is_button: bool,
}

impl SimpleArea {
    pub fn new(bounding_box: RenderBox, depth: u16) -> Self {
        Self { bounding_box, depth, alive: true, is_button: false }
    }

    pub fn new_button(bounding_box: RenderBox, depth: u16) -> Self {
        Self { bounding_box, depth, alive: true, is_button: true, }
    }
}
