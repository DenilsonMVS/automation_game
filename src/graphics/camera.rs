
use nalgebra_glm as glm;

use crate::graphics::input_state::{Actions, InputState};

const CAMERA_ZOOM_SENSITIVITY: f32 = 0.05;
const CAMERA_MOVEMENT_SENSITIVITY: f32 = 1.0;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    position: glm::Vec2,
    zoom: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: glm::vec2(0.0, 0.0),
            zoom: 1.0,
        }
    }

    pub fn get_view_matrix(&self) -> glm::Mat4 {
        let identity = glm::identity();
        let scaled = glm::scale(&identity, &glm::vec3(self.zoom, self.zoom, 1.0));
        let view = glm::translate(&scaled, &glm::vec3(-self.position.x, -self.position.y, 0.0));
        return view;
    }

    pub fn get_vp_matrix(&self, projection_matrix: &glm::Mat4) -> glm::Mat4 {
        projection_matrix * self.get_view_matrix()
    }

    pub fn update_movement(&mut self, input_state: &InputState) {
        let actions  = [
            (Actions::MoveBottom, glm::vec2( 0.0, -1.0)),
            (Actions::MoveLeft,   glm::vec2(-1.0,  0.0)),
            (Actions::MoveRight,  glm::vec2( 1.0,  0.0)),
            (Actions::MoveUp,     glm::vec2( 0.0,  1.0)),
        ];

        self.position += actions
            .iter()
            .fold(glm::vec2(0.0, 0.0),
                |prev, (action, value)| 
                    if input_state.get_actions()[*action] { prev + value } else { prev }
            ) / self.zoom * input_state.get_delta() * CAMERA_MOVEMENT_SENSITIVITY;
    }

    pub fn handle_scroll(&mut self, scroll: f32) {
        self.zoom *= f32::exp(-CAMERA_ZOOM_SENSITIVITY * scroll)
    }

    pub fn get_position(&self) -> glm::Vec2 {
        self.position
    }

    pub fn get_zoom(&self) -> f32 {
        self.zoom
    }

    pub fn get_mouse_world_pos(&self, mouse_position: glm::Vec2) -> glm::Vec2 {
        mouse_position / self.zoom + self.position
    }
}
