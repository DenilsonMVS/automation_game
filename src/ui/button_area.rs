use crate::graphics::renderer::positioning::RenderBox;

pub struct ButtonArea {
    pub bounding_box: RenderBox,
}

impl ButtonArea {
    pub fn new(bounding_box: RenderBox ) -> Self {
        Self { bounding_box }
    }
}
