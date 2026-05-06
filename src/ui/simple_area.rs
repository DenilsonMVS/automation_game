use winit::window::CursorIcon;

use crate::graphics::renderer::positioning::RenderBox;

#[derive(Debug, Clone, Copy)]
pub struct SimpleArea {
    pub bounding_box: RenderBox,
    pub depth: u16,
    pub cursor: CursorIcon,
    pub is_scroll: bool,
}

impl SimpleArea {
    pub fn new(bounding_box: RenderBox, depth: u16) -> Self {
        Self { bounding_box, depth, cursor: CursorIcon::Default, is_scroll: false }
    }

    pub fn with_cursor(mut self, cursor: CursorIcon) -> Self {
        self.cursor = cursor;
        self
    }

    pub fn activate_scroll(mut self) -> Self {
        self.is_scroll = true;
        self
    }
}
