use crate::{graphics::renderer::{fonts::{TextAlign, TextRenderConfig}, positioning::{PositionMode, RenderBox}}, ui::UiElement};
use nalgebra_glm as glm;

pub struct Text {
    bbox: RenderBox,
    content: String,
    color: glm::U8Vec4,
    align: TextAlign,
}

impl UiElement for Text {
    fn update(&mut self,
        parent_bbox: &RenderBox,
        parent_depth: u16,
        ctx: &mut super::UiState,
    ) {
        let depth = parent_depth - 1;
        let bbox = parent_bbox.sub_box(
            self.bbox.get_top_left(),
            self.bbox.get_size(),
            PositionMode::TopLeft,
            PositionMode::TopLeft,
        );
        
        ctx.renderer.font_renderer.push(
            &TextRenderConfig::new(
                &[(&self.content, self.color)],
                bbox.get_center(),
                bbox.get_size().y,
            )
            .with_align(self.align)
            .with_depth(depth)
            .with_mode(PositionMode::Center),
            ctx.font, 1,
        );
    }
}