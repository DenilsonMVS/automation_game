
use nalgebra_glm as glm;

use crate::graphics::renderer::positioning::{PositionMode, RenderBox};

pub struct Layout {
    pub offset: glm::Vec2,
    pub size: glm::Vec2,
    pub parent_anchor: PositionMode,
    pub anchor: PositionMode,
}

impl Layout {
    pub fn new(size: glm::Vec2) -> Self {
        Self {
            offset: glm::vec2(0.0, 0.0),
            size,
            parent_anchor: PositionMode::TopLeft,
            anchor: PositionMode::TopLeft,
        }
    }

    pub fn with_offset(mut self, offset: glm::Vec2) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_general_anchor(mut self, anchor: PositionMode) -> Self {
        self.anchor = anchor;
        self.parent_anchor = anchor;
        self
    }

    pub fn with_anchor(mut self, anchor: PositionMode) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn with_parent_anchor(mut self, anchor: PositionMode) -> Self {
        self.parent_anchor = anchor;
        self
    }

    pub fn position(&self, parent_bbox: &RenderBox) -> RenderBox {
        parent_bbox.sub_box(
            self.offset,
            self.size,
            self.anchor,
            self.parent_anchor,
        )
    }
}
