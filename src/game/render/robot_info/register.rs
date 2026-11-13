use crate::{game::{logic::top_level::instruction::UnpackedInstruction, render::robot_info::get_secondary_background_color}, graphics::renderer::{fonts::{TextAlign, TextRenderConfig}, positioning::RenderBox}, ui::{UiElement, UiState}};

pub struct Register {
    idx: usize,
}

impl Register {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}

fn u8_to_hex(val: u8) -> [u8; 2] {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
    [
        HEX_CHARS[(val >> 4) as usize],
        HEX_CHARS[(val & 0x0f) as usize],
    ]
}

impl UiElement for Register {
    fn update(&mut self,
        parent_bbox: &RenderBox,
        parent_depth: u16,
        ctx: &mut UiState,
    ) {
        const SPACING: f32 = 0.005;
        
        let depth = parent_depth - 1;
        ctx.renderer.simple_shape_renderer.send_render_box(
            parent_bbox,
            get_secondary_background_color(),
            depth, 1,
        );

        let depth = depth - 1;
        let value = ctx.watched_robot.as_ref()
            .map(|robot| robot.computer.get_data().get(self.idx))
            .flatten()
            .cloned()
            .unwrap_or(0);

        let bytes = u8_to_hex(value);
        let hex = unsafe { std::str::from_utf8_unchecked(&bytes) };

        ctx.renderer.font_renderer.push(&TextRenderConfig::new(
            &[(&hex, UnpackedInstruction::get_literal_color())],
            parent_bbox.get_center(),
            parent_bbox.get_size().y - SPACING * 2.0,
        ).with_align(TextAlign::Center).with_depth(depth), ctx.font, 1);
    }
}
