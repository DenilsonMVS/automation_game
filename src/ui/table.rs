use crate::{graphics::renderer::positioning::{PositionMode, RenderBox}, ui::{ListeningArea, Ui, UiElement, UiNode, UiState, layout::Layout}};
use nalgebra_glm as glm;
use winit::window::CursorIcon;

pub struct Table {
    pub offset: glm::Vec2,
    pub anchor: PositionMode,
    pub parent_anchor: PositionMode,

    pub lines: usize,
    pub item_size: glm::Vec2,
    pub gap: f32,
    pub color: glm::U8Vec4,
    pub children: Vec<UiNode>,
    id: u64,
}

impl Table {
    pub fn new(
        lines: usize,
        item_size: glm::Vec2,
        gap: f32,
        color: glm::U8Vec4,
        children: Vec<UiNode>,
        ui: &mut Ui,
    ) -> Self {
        Self {
            lines, item_size, gap, color, children,
            offset: glm::vec2(0.0, 0.0),
            anchor: PositionMode::TopLeft,
            parent_anchor: PositionMode::TopLeft,
            id: ui.new_id(),
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
}

impl UiElement for Table {
    fn update(&mut self,
        parent_bbox: &RenderBox,
        parent_depth: u16,
        ctx: &mut UiState,
    ) {
        let depth = parent_depth - 1;
        let columns = (self.children.len() + self.lines - 1) / self.lines;
        let size = glm::vec2(
            columns as f32 * (self.item_size.x + self.gap) + self.gap,
            self.lines as f32 * (self.item_size.y + self.gap) + self.gap,
        );
        let layout = Layout::new(size)
            .with_offset(self.offset)
            .with_anchor(self.anchor)
            .with_parent_anchor(self.parent_anchor);
        let bbox = layout.position(parent_bbox);

        if self.color.w != 0 {
            ctx.renderer.simple_shape_renderer.send_render_box(&bbox, self.color, depth, 1);
        }

        ctx.ui.push_area(ListeningArea { bbox, id: self.id, depth, cursor: CursorIcon::Default });
        for (i, line) in self.children.chunks_mut(columns).enumerate() {
            for (j, child) in line.iter_mut().enumerate() {
                let offset = glm::vec2(
                    j as f32 * (self.item_size.x + self.gap) + self.gap,
                    -(i as f32) * (self.item_size.y + self.gap) - self.gap
                );

                let bbox = bbox.sub_box(
                    offset, self.item_size,
                    PositionMode::TopLeft, PositionMode::TopLeft,
                );
                let depth = depth - 1;

                child.update(&bbox, depth, ctx);
            }
        }
    }
}
