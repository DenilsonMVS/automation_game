use winit::window::CursorIcon;

use crate::graphics::renderer::positioning::RenderBox;

#[derive(Debug, Clone, Copy)]
pub struct SimpleArea {
    pub bounding_box: RenderBox,
    pub depth: u16,
    pub alive: bool,
    pub cursor: CursorIcon,
}

impl SimpleArea {
    pub fn new(bounding_box: RenderBox, depth: u16) -> Self {
        Self { bounding_box, depth, alive: true, cursor: CursorIcon::Default }
    }

    pub fn with_cursor(mut self, cursor: CursorIcon) -> Self {
        self.cursor = cursor;
        self
    }
}
