use crate::graphics::renderer::{positioning::{PositionMode, RenderBox}, scissor_group::ScissorStack, simple_shape_renderer::SimpleShapeRenderer};
use nalgebra_glm as glm;

pub fn send(bbox: &RenderBox, depth: u16, color: glm::U8Vec4, simple_shape_renderer: &mut SimpleShapeRenderer, groups: &mut ScissorStack) {
    let size = bbox.get_size();
    let line_width = size.y * 0.2;
    simple_shape_renderer.send_rectangle(
        bbox.get_center() - glm::vec2(0.2 * size.x, 0.0),
        glm::vec2(size.x * 0.8, line_width),
        PositionMode::CenterLeft,
        color, depth, None, groups,
    );

    simple_shape_renderer.send_symmetrical_triangle(
        bbox.get_top_left() + glm::vec2(0.0, -size.y * 0.5),
        bbox.get_top_left() + glm::vec2(size.x * 0.3, -size.y * 0.5),
        size.y, color, depth, None, groups,
    );
}
