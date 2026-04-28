use std::{mem, time::Instant};

use crate::{game::{main_scene::MainScene, scene::EventType}, graphics::{camera::Camera, renderer::positioning::RenderBox}, ui::{drag_area_manager::DragAreaManager, multi_area_conflict_resolver::resolve_multi_area_conflict, simple_area_manager::SimpleManager}};
use nalgebra_glm as glm;
use winit::{event::{ElementState, KeyEvent}, keyboard::{KeyCode, PhysicalKey}, window::Window};

pub mod simple_area;
pub mod simple_area_manager;
pub mod drag_area_manager;
pub mod multi_area_conflict_resolver;

const CAMERA_ZOOM_SENSITIVITY: f32 = 0.05;
const CAMERA_MOVEMENT_SENSITIVITY: f32 = 1.0;

pub struct UiContext<'a, 'b> {
    pub window: &'a Window,
    pub scene: &'a mut MainScene<'b>,
}

fn map_key_to_index(key: KeyCode) -> Option<usize> {
    match key {
        KeyCode::KeyW => Some(0),
        KeyCode::KeyA => Some(1),
        KeyCode::KeyS => Some(2),
        KeyCode::KeyD => Some(3),
        _ => None,
    }
}

pub struct Ui {
    ndc_mouse_position: glm::Vec2,
    previous_ndc_mouse_position: glm::Vec2,
    window_size: glm::Vec2,
    current_time: Instant,
    previous_frame_time: Instant,
    mouse_pressed: bool,
    mouse_released: bool,
    pub camera: Camera,
    pressed_keys: [bool; 4],

    pub simple_area_manager: SimpleManager,
    pub drag_area_manager: DragAreaManager,

    pub events: Vec<EventType>,
}

impl Ui {
    pub fn new(window_size: glm::Vec2) -> Self {
        let current_time = Instant::now();
        Self {
            ndc_mouse_position: glm::vec2(0.0, 0.0),
            previous_ndc_mouse_position: glm::vec2(0.0, 0.0),
            window_size,
            current_time,
            previous_frame_time: current_time,
            
            mouse_pressed: false,
            mouse_released: false,

            camera: Camera::new(glm::vec2(0.0, 0.0), 1.0),
            pressed_keys: [false; 4],

            simple_area_manager: SimpleManager::new(),
            drag_area_manager: DragAreaManager::new(),

            events: vec![],
        }
    }

    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }

    pub fn on_key_event(&mut self, event: &KeyEvent) {
        match event.physical_key {
            PhysicalKey::Code(code) =>
                if let Some(idx) = map_key_to_index(code) {
                    self.pressed_keys[idx] = event.state == ElementState::Pressed;
                },
            _ => (),
        };
    }

    pub fn get_ndc_render_box(&self) -> RenderBox {
        RenderBox {
            min: self.screen_to_ndc_position(glm::vec2(0.0, 0.0)),
            max: self.screen_to_ndc_position(glm::vec2(1.0, 1.0)),
        }
    }

    pub fn update_mouse_position(&mut self, screen_position: glm::Vec2) {
        self.ndc_mouse_position = self.screen_to_ndc_position(screen_position);
    }

    pub fn on_mouse_scroll(&mut self, scroll: f32) {
        self.camera = Camera::new(
            self.camera.get_position(),
            self.camera.get_zoom() * (scroll * CAMERA_ZOOM_SENSITIVITY).exp(),
        );
    }

    pub fn on_render_start(&mut self, context: &mut UiContext) {
        self.previous_frame_time = self.current_time;
        self.current_time = Instant::now();

        let camera_movement_offsets = [
            glm::vec2( 0.0,  1.0),
            glm::vec2(-1.0,  0.0),
            glm::vec2( 0.0, -1.0),
            glm::vec2( 1.0,  0.0),
        ];

        for (idx, state) in self.pressed_keys.iter().enumerate() {
            if *state {
                self.camera = Camera::new(
                    self.camera.get_position() + self.get_delta() * camera_movement_offsets[idx] / self.camera.get_zoom() * CAMERA_MOVEMENT_SENSITIVITY,
                    self.camera.get_zoom(),
                );
            }
        }

        let mut areas_containing_mouse = Vec::new();
        self.drag_area_manager.on_frame_start(
            &mut areas_containing_mouse,
            self.ndc_mouse_position,
        );
        self.simple_area_manager.on_frame_start(
            &mut areas_containing_mouse,
            self.ndc_mouse_position,
        );

        resolve_multi_area_conflict(
            self.mouse_pressed,
            self.mouse_released,
            self.ndc_mouse_position,
            self.previous_ndc_mouse_position,
            &areas_containing_mouse,
            &mut self.events,
            &mut self.drag_area_manager,
            context.window,
        );
    }

    pub fn mouse_pressed(&mut self) {
        self.mouse_pressed = true;
    }

    pub fn mouse_released(&mut self) {
        self.mouse_released = true;
    }

    pub fn on_render_end(&mut self) {
        self.drag_area_manager.on_frame_end();
        self.simple_area_manager.on_frame_end();

        self.previous_ndc_mouse_position = self.ndc_mouse_position;
        self.mouse_pressed = false;
        self.mouse_released = false;
    }

    pub fn update_window_size(&mut self, window_size: glm::Vec2) {
        self.window_size = window_size;
    }

    pub fn get_delta(&self) -> f32 {
        self.current_time.duration_since(self.previous_frame_time).as_secs_f32()
    }

    pub fn get_projection_matrix(&self) -> glm::Mat4 {
        let window_size = self.window_size;
        if window_size.x > window_size.y {
            let ratio = window_size.x / window_size.y;
            glm::ortho(-ratio, ratio, -1.0, 1.0, -1.0, 1.0)
        } else {
            let ratio = window_size.y / window_size.x;
            glm::ortho(-1.0, 1.0, -ratio, ratio, -1.0, 1.0)
        }
    }

    pub fn screen_to_ndc_position(&self, position: glm::Vec2) -> glm::Vec2 {
        let window_size = self.window_size;
        let normalized = position.component_div(&window_size);
        let inverted = glm::vec2(normalized.x * 2.0 - 1.0, 1.0 - 2.0 * normalized.y);
        if window_size.x > window_size.y {
            let ratio = window_size.x / window_size.y;
            glm::vec2(inverted.x * ratio, inverted.y)
        } else {
            let ratio = window_size.y / window_size.x;
            glm::vec2(inverted.x, inverted.y * ratio)
        }
    }

    pub fn take_events(&mut self) -> Vec<EventType> {
        self.events.push(EventType::FinalEvent);
        mem::take(&mut self.events)
    }
}
