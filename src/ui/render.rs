use crate::{graphics::renderer::{fonts::{Font, TextRenderConfig}, positioning::{PositionMode, RenderBox}, simple_shape_renderer::SimpleShapeRenderer}, ui::{DirectionInfo, UiElement}};
use nalgebra_glm as glm;

fn ui_coord_inversion(vec: glm::Vec2) -> glm::Vec2 {
    glm::vec2(vec.x, -vec.y)
}
 
impl RenderBox {
    fn sub_box(&self, position: glm::Vec2, size: glm::Vec2) -> Self {
        let top_left = self.get_top_left();
        let offset_applied = top_left + ui_coord_inversion(position);
        let size_applied = offset_applied + ui_coord_inversion(size);
        Self {
            min: glm::vec2(offset_applied.x, size_applied.y),
            max: glm::vec2(size_applied.x, offset_applied.y),
        }
    }

    fn cut(&self, cut: &DirectionInfo<f32>) -> Self {
        Self {
            min: self.min + glm::vec2(cut.left, cut.bottom),
            max: self.max - glm::vec2(cut.right, cut.top),
        }
    }
}

struct RenderPassContext<'pass, 'data> {
    font: &'pass mut Font<'data>,
    simple_shape_renderer: &'pass mut SimpleShapeRenderer<'data>,
    transparent_boxes: Vec<(RenderBox, glm::U8Vec4, u16)>,
}

impl UiElement {
    pub fn render<'data>(
        &self,
        font: &mut Font<'data>,
        simple_shape_renderer: &mut SimpleShapeRenderer<'data>,
        parent_render_box: &RenderBox,
    ) {
        let mut context = RenderPassContext {
            font, simple_shape_renderer,
            transparent_boxes: vec![],
        };

        self.render_rec(&mut context, 100, parent_render_box);

        for (render_box, color, depth) in context.transparent_boxes {
            simple_shape_renderer.send_render_box(
                &render_box,
                color, depth, 1,
            );
        }
    }

    fn decide_box_rendering(
        context: &mut RenderPassContext,
        render_box: &RenderBox,
        color: glm::U8Vec4,
        depth: u16,
    ) {
        if color.w != 0 {
            if color.w != 255 {
                context.transparent_boxes.push(
                    (*render_box, color, depth));
            } else {
                context.simple_shape_renderer.send_render_box(
                    &render_box,
                    color, depth, 1,
                );
            }
        }
    }

    fn render_rec(&self, context: &mut RenderPassContext, parent_depth: u16, parent_render_box: &RenderBox) {
        let current_depth = parent_depth - 2;
        let border_depth = parent_depth - 1;
        match self {
            Self::Box(ui_box) => {
                let outer_render_box = parent_render_box.sub_box(ui_box.position, ui_box.size);
                let margin_render_box = outer_render_box.cut(&ui_box.margin);
                let border_render_box = margin_render_box.cut(&ui_box.border);
                let padding_render_box = border_render_box.cut(&ui_box.padding);
                
                for child in ui_box.children.iter() {
                    child.render_rec(context, current_depth, &padding_render_box);
                }

                Self::decide_box_rendering(context, &border_render_box, ui_box.background_color, current_depth);
                

                if    
                    ui_box.border.bottom > 0.0 || ui_box.border.top > 0.0 ||
                    ui_box.border.left > 0.0 || ui_box.border.right > 0.0
                {
                    Self::decide_box_rendering(context, &margin_render_box, ui_box.border_color, border_depth);
                }
            },
            Self::Text(text) => {
                let max_line_width = parent_render_box.max.x - parent_render_box.min.x;
                context.font.add_text(
                    &TextRenderConfig::new(
                        &[(&text.content, text.color)],
                        parent_render_box.get_top_left(),
                        text.line_height,
                    )
                    .set_align(text.align)
                    .set_mode(PositionMode::TopLeft)
                    .set_depth(current_depth)
                    .set_max_line_width(max_line_width),
                );
            },
        }
    }
}