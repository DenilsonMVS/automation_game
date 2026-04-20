use crate::{graphics::renderer::{fonts::{Font, TextRenderConfig}, positioning::{PositionMode, RenderBox}, simple_shape_renderer::SimpleShapeRenderer}, ui::{DirectionInfo, Div, UiElement}};
use nalgebra_glm as glm;

pub(super) fn ui_coord_inversion(vec: glm::Vec2) -> glm::Vec2 {
    glm::vec2(vec.x, -vec.y)
}
 
impl RenderBox {
    pub(super) fn sub_div(&self, position: glm::Vec2, size: glm::Vec2) -> Self {
        let top_left = self.get_top_left();
        let offset_applied = top_left + ui_coord_inversion(position);
        let size_applied = offset_applied + ui_coord_inversion(size);
        Self {
            min: glm::vec2(offset_applied.x, size_applied.y),
            max: glm::vec2(size_applied.x, offset_applied.y),
        }
    }

    pub(super) fn cut(&self, cut: &DirectionInfo<f32>) -> Self {
        Self {
            min: self.min + glm::vec2(cut.left, cut.bottom),
            max: self.max - glm::vec2(cut.right, cut.top),
        }
    }

    pub(super) fn apply_offset(&self, offset: glm::Vec2) -> Self {
        Self {
            min: self.min + offset,
            max: self.max + offset,
        }
    }
}

struct RenderPassContext<'pass, 'data> {
    font: &'pass mut Font<'data>,
    simple_shape_renderer: &'pass mut SimpleShapeRenderer<'data>,
    transparent_divs: Vec<(RenderBox, glm::U8Vec4, u16)>,
}

impl<C> UiElement<C> {
    pub fn send<'data>(
        &self,
        font: &mut Font<'data>,
        simple_shape_renderer: &mut SimpleShapeRenderer<'data>,
        parent_render_div: &RenderBox,
    ) {
        let mut context = RenderPassContext {
            font, simple_shape_renderer,
            transparent_divs: vec![],
        };

        self.send_rec(&mut context, 100, parent_render_div);

        for (render_div, color, depth) in context.transparent_divs {
            simple_shape_renderer.send_render_box(
                &render_div,
                color, depth, 1,
            );
        }
    }

    fn test_transparent(
        context: &mut RenderPassContext,
        render_div: &RenderBox,
        color: glm::U8Vec4,
        depth: u16,
    ) {
        if color.w != 0 && color.w != 255 {
            context.transparent_divs.push(
                (*render_div, color, depth));
        }
    }

    fn test_opaque(
        context: &mut RenderPassContext,
        render_div: &RenderBox,
        color: glm::U8Vec4,
        depth: u16,
    ) {
        if color.w == 255 {
            context.simple_shape_renderer.send_render_box(
                &render_div,
                color, depth, 1,
            );
        }
    }

    fn has_border(border: &DirectionInfo<f32>) -> bool {
        border.bottom > 0.0 || border.top > 0.0 ||
        border.left > 0.0 || border.right > 0.0
    }

    fn send_div(
        ui_div: &Div<C>,
        context: &mut RenderPassContext,
        parent_depth: u16,
        parent_render_div: &RenderBox,
        custom_border_color: Option<glm::U8Vec4>,
        custom_background_color: Option<glm::U8Vec4>,
    ) {
        let border_color = custom_border_color.unwrap_or(ui_div.border_color);
        let background_color = custom_background_color.unwrap_or(ui_div.background_color);

        let current_depth = parent_depth - 2;
        let border_depth = parent_depth - 1;

        let outer_render_div = parent_render_div.sub_div(ui_div.position, ui_div.size);
        let margin_render_div = outer_render_div.cut(&ui_div.margin);
        let border_render_div = margin_render_div.cut(&ui_div.border);
        let padding_render_div = border_render_div.cut(&ui_div.padding);
        
        if Self::has_border(&ui_div.border) {
            Self::test_transparent(context, &margin_render_div, border_color, border_depth);
        }
        Self::test_transparent(context, &border_render_div, background_color, current_depth);

        for child in ui_div.children.iter() {
            child.send_rec(context, current_depth, &padding_render_div);
        }

        Self::test_opaque(context, &border_render_div, background_color, current_depth);
        if Self::has_border(&ui_div.border) {
            Self::test_opaque(context, &margin_render_div, border_color, border_depth);
        }
    }

    fn send_rec(&self, context: &mut RenderPassContext, parent_depth: u16, parent_render_div: &RenderBox) {
        match self {
            Self::Div(ui_div) =>
                Self::send_div(ui_div, context, parent_depth, parent_render_div, None, None),
            Self::Text(text) => {
                let max_line_width = parent_render_div.max.x - parent_render_div.min.x;
                context.font.add_text(
                    &TextRenderConfig::new(
                        &[(&text.content, text.color)],
                        parent_render_div.get_top_left(),
                        text.line_height,
                    )
                    .set_align(text.align)
                    .set_mode(PositionMode::TopLeft)
                    .set_depth(parent_depth - 1)
                    .set_max_line_width(max_line_width),
                );
            },
            Self::Button(button) =>
                if button.is_hovered {
                    Self::send_div(
                        &button.container,
                        context, parent_depth,
                        parent_render_div,
                        Some(button.hover_border_color),
                        Some(button.hover_color),
                    )
                } else {
                    Self::send_div(
                        &button.container,
                        context, parent_depth,
                        parent_render_div,
                        None,
                        None,
                    )
                },
            Self::DragBox(drag_box) =>
                Self::send_div(
                    &drag_box.container,
                    context,
                    parent_depth,
                    &parent_render_div.apply_offset(drag_box.offset),
                    None,
                    None,
                ),
        }
    }
}