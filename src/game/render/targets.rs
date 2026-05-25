use crate::{MainContext, game::{logic::{target::Targets, world::CHUNK_SIZE}, render::robot::ROBOT_DEPTH}, graphics::renderer::{positioning::PositionMode}};
use nalgebra_glm as glm;

fn get_target_color() -> glm::U8Vec4 {
    glm::vec4(60, 70, 220, 100)
}

pub fn get_target_text_color() -> glm::U8Vec4 {
    glm::vec4(60, 70, 220, 255)
}

impl Targets {
    pub fn send(&self, chunk_id: glm::IVec2, context: &mut MainContext) {
        let chunk_offset = chunk_id.map(|x| (x * CHUNK_SIZE as i32) as f32);
        for target in self.get_targets().iter() {
            let pos = chunk_offset + target.map(|x| x as f32);
            context.simple_shape_renderer.send_rectangle(
                pos,
                glm::vec2(0.95, 0.95),
                PositionMode::Center,
                get_target_color(),
                ROBOT_DEPTH + 10,
                Some(context.ui.get_camera()),
                context.groups,
            );
        }
    }
}
