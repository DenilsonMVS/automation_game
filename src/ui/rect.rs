use crate::{graphics::renderer::positioning::RenderBox, ui::{ListeningArea, UiElement, UiNode, UiState, layout::Layout}};
use nalgebra_glm as glm;
use winit::window::CursorIcon;

pub struct Rect {
    pub layout: Layout,
    pub color: glm::U8Vec4,
    pub id: u64,
    pub children: Vec<UiNode>,
}

impl Rect {
    pub fn new(
        layout: Layout,
        color: glm::U8Vec4,
        id: u64,
        children: Vec<UiNode>,
    ) -> Self {
        Self {
            layout, color, id, children,
        }
    }
}

impl UiElement for Rect {
    fn update(&mut self,
        parent_bbox: &RenderBox,
        parent_depth: u16,
        ctx: &mut UiState,
    ) {
        let depth = parent_depth - 1;
        let bbox = self.layout.position(parent_bbox);

        if self.color.w != 0 {
            ctx.renderer.simple_shape_renderer.send_render_box(&bbox, self.color, depth, 1);
        }

        ctx.ui.push_area(ListeningArea { bbox, id: self.id, depth, cursor: CursorIcon::Default });
        for el in self.children.iter_mut() {
            el.update(&bbox, depth, ctx);
        }
    }
}
