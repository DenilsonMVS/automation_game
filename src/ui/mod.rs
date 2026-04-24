use std::time::Instant;

use crate::{game::main_scene::MainScene, graphics::{camera::Camera, renderer::positioning::RenderBox}, stable_index_vector::StableIndexVector, ui::{button_area::ButtonArea, drag_area::DragArea}};
use nalgebra_glm as glm;
use winit::{event::{ElementState, KeyEvent}, keyboard::{KeyCode, PhysicalKey}, window::{Cursor, CursorIcon, Window}};

pub mod button_area;
pub mod drag_area;

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
    button_areas: Vec<ButtonArea>,
    ndc_mouse_position: glm::Vec2,
    previous_ndc_mouse_position: glm::Vec2,
    window_size: glm::Vec2,
    current_time: Instant,
    previous_frame_time: Instant,
    mouse_pressed: bool,
    mouse_released: bool,
    camera: Camera,
    pressed_keys: [bool; 4],

    selected_drag_area: Option<usize>,
    drag_areas: StableIndexVector<DragArea>,
}

impl Ui {
    pub fn new(window_size: glm::Vec2) -> Self {
        let current_time = Instant::now();
        Self {
            button_areas: vec![],
            ndc_mouse_position: glm::vec2(0.0, 0.0),
            previous_ndc_mouse_position: glm::vec2(0.0, 0.0),
            window_size,
            current_time,
            previous_frame_time: current_time,
            
            mouse_pressed: false,
            mouse_released: false,

            camera: Camera::new(glm::vec2(0.0, 0.0), 1.0),
            pressed_keys: [false; 4],

            selected_drag_area: None,
            drag_areas: StableIndexVector::new(),
        }
    }

    pub fn push_drag_area(&mut self, drag_area: DragArea) -> usize {
        self.drag_areas.push(drag_area)
    }

    pub fn remove_drag_area(&mut self, id: usize) -> Option<DragArea> {
        self.drag_areas.remove(id)
    }

    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }

    pub fn on_key_event(&mut self, event: KeyEvent) {
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

    pub fn push_button_area(&mut self, button_area: ButtonArea) {
        self.button_areas.push(button_area);
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

    fn handle_drag_areas(&mut self, context: &mut UiContext) {

        if self.selected_drag_area.is_some() {
            context.window.set_cursor(Cursor::Icon(CursorIcon::Grabbing));
        } else {
            for drag_area in self.drag_areas.iter_unordered() {
                if drag_area.get_current_position().is_point_inside(self.ndc_mouse_position) {
                    context.window.set_cursor(Cursor::Icon(CursorIcon::Grab));
                    break;
                }
            }
        }

        if self.selected_drag_area.is_none() && self.mouse_pressed {
            // Testar para caso de podermos selecionar uma drag area

            for (id, drag_area) in self.drag_areas.iter_enumerate_unordered() {
                if drag_area.get_current_position().is_point_inside(self.ndc_mouse_position) {
                    self.selected_drag_area = Some(id);
                    drag_area.on_grab(context, self);
                    break;
                }
            }

        } else if
            self.mouse_released &&
            let Some(id) = self.selected_drag_area &&
            let Some(drag_area) = self.drag_areas.get(id)
        {
            // Soltamos a drag area
            self.selected_drag_area = None;
            drag_area.on_release(context, self);
        } else if
            let Some(id) = self.selected_drag_area &&
            self.ndc_mouse_position != self.previous_ndc_mouse_position
        {
            // Movemos a drag area
            if let Some(drag_area) = self.drag_areas.get_mut(id) {
                let diff = self.ndc_mouse_position - self.previous_ndc_mouse_position;
                drag_area.update_offset(diff);
            }

            if let Some(drag_area) = self.drag_areas.get(id) {
                drag_area.on_move(context, self);
            }
        }
    }

    pub fn get_drag_area(&self, idx: usize) -> Option<&DragArea> {
        self.drag_areas.get(idx)
    }

    pub fn on_render_start(&mut self, context: &mut UiContext) {
        self.previous_frame_time = self.current_time;
        self.current_time = Instant::now();
        context.window.set_cursor(Cursor::Icon(CursorIcon::Default));

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

        for button_area in self.button_areas.iter() {
            button_area.test(self.ndc_mouse_position, context, self.mouse_pressed, self);
        }

        self.handle_drag_areas(context);

        self.button_areas.clear();
    }

    pub fn mouse_pressed(&mut self) {
        self.mouse_pressed = true;
    }

    pub fn mouse_released(&mut self) {
        self.mouse_released = true;
    }

    pub fn on_render_end(&mut self) {
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
}
