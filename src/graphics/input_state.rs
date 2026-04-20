
use std::time::Instant;
use enum_map::{Enum, EnumMap};
use nalgebra_glm as glm;
use winit::{event::ElementState, keyboard::{KeyCode, PhysicalKey}};
use crate::graphics::renderer::positioning::RenderBox;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum Actions {
    MoveUp,
    MoveRight,
    MoveLeft,
    MoveBottom,
}

#[derive(Debug, Clone, Copy)]
struct UnitState {
    window_size: glm::Vec2,
    mouse_screen_position: glm::Vec2,
    current_time: Instant,
    actions: EnumMap<Actions, bool>,
}

#[derive(Debug, Clone, Copy)]
pub struct InputState {
    current_frame: UnitState,
    previous_frame: UnitState,
}

impl InputState {
    pub fn new(window_size: glm::Vec2) -> Self {
        let now = std::time::Instant::now();
        let actions = enum_map::enum_map! {
            _ => false
        };
        Self {
            current_frame: UnitState {
                window_size,
                mouse_screen_position: glm::vec2(0.0, 0.0),
                current_time: now,
                actions,
            },
            previous_frame: UnitState {
                window_size,
                mouse_screen_position: glm::vec2(0.0, 0.0),
                current_time: now,
                actions,
            },
        }
    }

    pub fn get_actions(&self) -> &EnumMap<Actions, bool> {
        &self.current_frame.actions
    }

    pub fn update_key(&mut self, key: KeyCode, state: ElementState) {
        match key {
            KeyCode::KeyW => self.current_frame.actions[Actions::MoveUp    ] = state.is_pressed(),
            KeyCode::KeyA => self.current_frame.actions[Actions::MoveLeft  ] = state.is_pressed(),
            KeyCode::KeyS => self.current_frame.actions[Actions::MoveBottom] = state.is_pressed(),
            KeyCode::KeyD => self.current_frame.actions[Actions::MoveRight ] = state.is_pressed(),
            _ => {},
        };
    }

    pub fn update_window_size(&mut self, window_size: glm::Vec2) {
        self.current_frame.window_size = window_size;
    }

    pub fn update_mouse_position(&mut self, mouse_screen_position: glm::Vec2) {
        self.current_frame.mouse_screen_position = mouse_screen_position;
    }

    pub fn next_frame(&mut self) {
        let now = std::time::Instant::now();
        self.previous_frame = self.current_frame;
        self.current_frame.current_time = now;
    }

    pub fn get_delta(&self) -> f32 {
        self.current_frame.current_time.duration_since(self.previous_frame.current_time).as_secs_f32()
    }

    pub fn get_projection_matrix(&self) -> glm::Mat4 {
        let window_size = self.current_frame.window_size;
        if window_size.x > window_size.y {
            let ratio = window_size.x / window_size.y;
            glm::ortho(-ratio, ratio, -1.0, 1.0, -1.0, 1.0)
        } else {
            let ratio = window_size.y / window_size.x;
            glm::ortho(-1.0, 1.0, -ratio, ratio, -1.0, 1.0)
        }
    }

    pub fn get_root_ui_render_box(&self) -> RenderBox {
        let window_size = self.current_frame.window_size;
        let ratio = window_size.x / window_size.y;
        if window_size.x > window_size.y {
            RenderBox {
                min: glm::vec2(-ratio, -1.0),
                max: glm::vec2( ratio,  1.0),
            }
        } else {
            RenderBox {
                min: glm::vec2(-1.0, -1.0 / ratio),
                max: glm::vec2( 1.0,  1.0 / ratio),
            }
        }
    }
}
