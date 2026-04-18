use crate::{graphics::renderer::{positioning::RenderBox, simple_shape_renderer::SimpleShapeRenderer}, ui::UiElement};
use nalgebra_glm as glm;

impl UiElement {
    pub fn render(&self, simple_shape_renderer: &mut SimpleShapeRenderer, parent_depth: u16, parent_top_left: glm::Vec2){
        let current_depth = parent_depth - 2;
        let border_depth = parent_depth - 1;
        match self {
            Self::Box(ui_box) => {
                let outer_render_box = RenderBox {
                    min: parent_top_left + ui_box.position - glm::vec2(0.0, ui_box.size.y),
                    max: parent_top_left + ui_box.position + glm::vec2(ui_box.size.x, 0.0),
                };

                let margin_render_box = RenderBox {
                    min: outer_render_box.min + glm::vec2(ui_box.margin.left, ui_box.margin.bottom),
                    max: outer_render_box.max - glm::vec2(ui_box.margin.right, ui_box.margin.top),
                };

                let border_render_box = RenderBox {
                    min: margin_render_box.min + glm::vec2(ui_box.border.left, ui_box.border.bottom),
                    max: margin_render_box.max - glm::vec2(ui_box.border.right, ui_box.border.top),
                };

                let padding_render_box = RenderBox {
                    min: border_render_box.min + glm::vec2(ui_box.padding.left, ui_box.padding.bottom),
                    max: border_render_box.max - glm::vec2(ui_box.padding.right, ui_box.padding.top),
                };

                
                for child in ui_box.children.iter() {
                    child.render(simple_shape_renderer, current_depth, padding_render_box.get_top_left());
                }

                simple_shape_renderer.send_render_box(
                    &border_render_box,
                    ui_box.background_color,
                    current_depth,
                    1,
                );

                if
                    ui_box.border.bottom > 0.0 || ui_box.border.top > 0.0 ||
                    ui_box.border.left > 0.0 || ui_box.border.right > 0.0
                {
                    simple_shape_renderer.send_render_box(
                        &margin_render_box,
                        ui_box.border_color,
                        border_depth,
                        1,
                    );
                } 
            },
            Self::Text(text) => {
                
            },
        }
    }
}