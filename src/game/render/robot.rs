use crate::graphics::renderer::{Renderer, fonts::{TextAlign, TextRenderConfig, font::Font}, positioning::{PositionMode, RenderBox}};
use nalgebra_glm as glm;

const ROBOT_SIZE: f32 = 0.8;
const WEEL_DIAMETER: f32 = 0.26;
const WEEL_DISTANCE: f32 = 0.37;
pub const ROBOT_DEPTH: u16 = 50000;
const WEELS_DEPTH: u16 = ROBOT_DEPTH + 1;
const DISPLAY_DEPTH: u16 = ROBOT_DEPTH - 1;

pub struct Robot {
    pub position: glm::Vec2,
    pub speed: glm::Vec2,
    pub display: Option<u8>,
}

pub fn send_robot(robot: &Robot, renderer: &mut Renderer, font: &mut Font, extra_time: f32) {
    let position = robot.position + robot.speed * extra_time;

    let edge_offsets = [
        glm::vec2(-1.0, -1.0),
        glm::vec2( 1.0, -1.0),
        glm::vec2( 1.0,  1.0),
        glm::vec2(-1.0,  1.0),
    ];

    let simple_shape = &mut renderer.simple_shape_renderer;

    for offset in edge_offsets {
        let bbox = RenderBox::new(
            position + offset * WEEL_DISTANCE,
            glm::vec2(WEEL_DIAMETER, WEEL_DIAMETER),
            PositionMode::Center,
        );

        simple_shape.send_ellipse(&bbox, get_weel_color(), WEELS_DEPTH, 0);
    }
    
    simple_shape.send_render_box(
        &get_bbox(position),
        get_robot_default_color(),
        ROBOT_DEPTH, 0,
    );

    if let Some(value) = robot.display {
        let value = value.to_string();
        renderer.font_renderer.push(
            &TextRenderConfig::new(
                &[(&value, glm::vec4(20, 45, 90, 255))],
                position,
                0.5,
            )
            .with_depth(DISPLAY_DEPTH)
            .with_max_line_width(0.5)
            .with_align(TextAlign::Center),
            font,
            0,
        );
    }
}


fn get_weel_color() -> glm::U8Vec4 {
    glm::vec4(45, 40, 40, 255)
}

fn get_robot_default_color() -> glm::U8Vec4 {
    glm::vec4(120, 200, 190, 255)
}

pub fn get_bbox(position: glm::Vec2) -> RenderBox {
    RenderBox::new(
        position,
        glm::vec2(ROBOT_SIZE, ROBOT_SIZE),
        PositionMode::Center,
    )
}
