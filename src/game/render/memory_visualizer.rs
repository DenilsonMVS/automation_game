use nalgebra_glm as glm;
use crate::{game::{logic::{computer::MEMORY_SIZE, instruction::{RegisterAddress, UnpackedInstruction}}, render::robot_info::{get_background_color, get_secondary_background_color}}, graphics::renderer::{fonts::{Font, TextAlign, TextRenderConfig}, positioning::{PositionMode, RenderBox}, scissor_group::ScissorStack, simple_shape_renderer::SimpleShapeRenderer}};

#[derive(Debug, Clone, Copy)]
pub struct SpecialHighlight {
    pub r1: Option<RegisterAddress>,
    pub r2: Option<RegisterAddress>,
    pub rd: Option<RegisterAddress>,
}

pub fn send(
    simple_shape_renderer: &mut SimpleShapeRenderer,
    font: &mut Font,
    groups: &mut ScissorStack,
    memory: &[u8; MEMORY_SIZE],
    bbox: &RenderBox,
    depth: u16,
    special_highlight: SpecialHighlight,
) {
    let gap = 0.005;
    let text_gap = 0.003;
    let padded_box = bbox.cut(gap);
    let unit_size = (padded_box.get_size() - glm::vec2(gap, gap) * 3.0) * 0.25;

    for (idx, value) in memory.iter().enumerate()  {
        let i = idx / 4;
        let j = idx % 4;
        let offset = glm::vec2(
            j as f32 * (unit_size.x + gap),
            i as f32 * (-(unit_size.y + gap)),
        );
        let pos = bbox.get_top_left() + offset + glm::vec2(gap, -gap);
        let bbox = RenderBox::new_simple(pos, PositionMode::TopLeft, unit_size);

        simple_shape_renderer.send_render_box(
            &bbox,
            get_background_color(), depth - 1,
            None, groups,
        );

        let opt = Some(idx as u16);
        let color = if opt == special_highlight.r1.map(|v| v.get_raw()) {
            UnpackedInstruction::get_reg1_color()
        } else if opt == special_highlight.r2.map(|v| v.get_raw()) {
            UnpackedInstruction::get_reg2_color()
        } else if opt == special_highlight.rd.map(|v| v.get_raw()) {
            UnpackedInstruction::get_dest_color()
        } else {
            glm::vec4(140, 140, 140, 255)
        };
        let text_bbox = bbox.cut(text_gap);
        let text = format!("{}", value);
        font.add_text(
            &TextRenderConfig::new(
                &[(&text, color)],
                text_bbox.get_top_left(),
                text_bbox.get_size().y,
            )
            .set_max_line_width(text_bbox.get_size().x)
            .set_align(TextAlign::Center)
            .set_depth(depth - 2)
            .set_mode(PositionMode::TopLeft),
            groups,
        );
    }

    simple_shape_renderer.send_render_box(
        bbox,
        get_secondary_background_color(), depth,
        None, groups,
    );
}
