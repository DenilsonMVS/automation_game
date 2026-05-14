use crate::{MainContext, game::{logic::{fabricator::{Fabricator, PHASE_AMOUNT}, robot::get_robot_default_color}, render::robot::{ROBOT_DEPTH, send_ghost}}, graphics::renderer::{positioning::{PositionMode, RenderBox}}};
use nalgebra_glm as glm;

const PHASE_INDICATOR_UNIT_SIZE: f32 = 0.05;
const PHASE_INDICATOR_GAP: f32 = 0.01;

impl Fabricator {
    pub(super) fn send(&self, context: &mut MainContext) {
        let fpos = self.position.map(|x| x as f32);
        let outer_indicator = RenderBox::new_simple(
            fpos,
            PositionMode::Center,
            glm::vec2(
                (PHASE_INDICATOR_UNIT_SIZE + PHASE_INDICATOR_GAP) * PHASE_AMOUNT as f32 * 0.5 + PHASE_INDICATOR_GAP,
                PHASE_INDICATOR_UNIT_SIZE * 2.0 + PHASE_INDICATOR_GAP * 3.0,
            )
        );

        if self.phase > 0 {
            let alpha = self.phase * 16;
            let robot_pos = self.position + self.output.get_offset();
            send_ghost(robot_pos.map(|x| x as f32), get_robot_default_color(), alpha as u8, context);
        }

        let unit_color = glm::vec4(240, 170, 20, 255);
        for phase in 0..self.phase {
            let j = phase % (PHASE_AMOUNT / 2);
            let i = phase / (PHASE_AMOUNT / 2);
            let pos =
                outer_indicator.get_top_left() +
                glm::vec2((PHASE_INDICATOR_UNIT_SIZE + PHASE_INDICATOR_GAP) * j as f32, 0.0) +
                glm::vec2(PHASE_INDICATOR_GAP, -PHASE_INDICATOR_GAP) +
                glm::vec2(0.0, -(i as f32) * (PHASE_INDICATOR_UNIT_SIZE + PHASE_INDICATOR_GAP));
            let bbox = RenderBox::new_simple(pos, PositionMode::TopLeft, glm::vec2(PHASE_INDICATOR_UNIT_SIZE, PHASE_INDICATOR_UNIT_SIZE));
            context.simple_shape_renderer.send_render_box(
                &bbox,
                unit_color,
                ROBOT_DEPTH - 2,
                Some(context.ui.get_camera()),
                context.groups,
            );
        }

        context.simple_shape_renderer.send_render_box(
            &outer_indicator,
            glm::vec4(0, 0, 0, 255),
            ROBOT_DEPTH - 1,
            Some(context.ui.get_camera()),
            context.groups,
        );

        context.simple_shape_renderer.send_render_box(
            &Self::get_bbox(self.position.map(|x| x as f32)),
            glm::vec4(100, 100, 100, 255),
            ROBOT_DEPTH,
            Some(context.ui.get_camera()),
            context.groups,
        );
    }
}