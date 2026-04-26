use crate::{graphics::renderer::positioning::RenderBox};

#[derive(Debug)]
pub struct DragArea {
    pub bounding_box: RenderBox,
}

impl DragArea {
    pub fn new(bounding_box: RenderBox) -> Self {
        Self {
            bounding_box: bounding_box,
        }
    }
}
