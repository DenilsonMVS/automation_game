
use glfw::{Action, Key, PWindow};
use nalgebra_glm as glm;

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

    pub fn handle_keyboard_input(&mut self, window: &PWindow, delta: f32) {
        let camera_keys = [
            (Key::W, glm::vec2( 0.0f32,  1.0f32)),
            (Key::A, glm::vec2(-1.0f32,  0.0f32)),
            (Key::S, glm::vec2( 0.0f32, -1.0f32)),
            (Key::D, glm::vec2( 1.0f32,  0.0f32)),
        ];

        for (key, change) in camera_keys {
            if window.get_key(key) == Action::Press {
                self.position += change / self.zoom * delta * CAMERA_MOVEMENT_SENSITIVITY;
            }
        }
    }

    pub fn handle_scroll(&mut self, scroll: f32) {
        self.zoom *= f32::exp(CAMERA_ZOOM_SENSITIVITY * scroll)
    }

    pub fn get_position(&self) -> glm::Vec2 {
        self.position
    }

    pub fn get_zoom(&self) -> f32 {
        self.zoom
    }
}
