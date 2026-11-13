
use std::time::Instant;

use nalgebra_glm as glm;
use winit::{event::ElementState, window::{CursorIcon, Window}};

use crate::{game::{logic::top_level::{instruction::Instruction, robot::RobotLogic}, render::robot_info::{InstructionBox, register::Register}}, graphics::{camera::Camera, renderer::{Renderer, fonts::font::Font, positioning::{RenderBox, get_screen_bbox, screen_position_to_ndc}}}, ui::{area_conflit_resolve::resolve_conflicts, rect::Rect, table::Table, text::Text}};
pub mod area_conflit_resolve;
pub mod rect;
pub mod text;
pub mod table;
pub mod layout;

pub trait UiElement {
    fn update(&mut self,
        parent_bbox: &RenderBox,
        parent_depth: u16,
        ctx: &mut UiState,
    );
}

pub enum UiNode {
    Rect(Rect),
    Text(Text),
    InstructionBox(InstructionBox),
    Instruction(Instruction),
    Register(Register),
    Table(Table),
}

impl UiElement for UiNode {
    fn update(&mut self,
        parent_bbox: &RenderBox,
        parent_depth: u16,
        ctx: &mut UiState,
    ) {
        match self {
            Self::Rect(rect) => rect.update(parent_bbox, parent_depth, ctx),
            Self::Text(text) => text.update(parent_bbox, parent_depth, ctx),
            Self::InstructionBox(ibox) => ibox.update(parent_bbox, parent_depth, ctx),
            Self::Instruction(inst) => inst.update(parent_bbox, parent_depth, ctx),
            Self::Register(reg) => reg.update(parent_bbox, parent_depth, ctx),
            Self::Table(tab) => tab.update(parent_bbox, parent_depth, ctx),
        }
    }
}

pub struct UiState<'a> {
    pub renderer: &'a mut Renderer,
    pub ui: &'a mut Ui,
    pub font: &'a mut Font,
    pub watched_robot: Option<RobotLogic>,
}


#[derive(Debug, Clone, Copy)]
pub struct ListeningArea {
    pub bbox: RenderBox,
    pub id: u64,
    pub depth: u16,
    pub cursor: CursorIcon,
}

#[derive(Debug)]
pub struct Ui {
    window_size: glm::Vec2,
    pub camera: Camera,
    next_id: u64,
    areas: Vec<ListeningArea>,

    mouse_position: glm::Vec2,
    previous_mouse_position: glm::Vec2,

    mouse_state: ElementState,
    previout_mouse_state: ElementState,

    previous_time: Instant,
    current_time: Instant,

    mouse_area_stack: Vec<u64>,
}

impl Ui {
    pub fn new(window_size: glm::Vec2) -> Self {
        let now = Instant::now();
        Self {
            window_size,
            camera: Camera::new(glm::vec2(0.0, 0.0), 1.0),
            next_id: 0,
            areas: vec![],
            mouse_position: glm::vec2(0.0, 0.0),
            previous_mouse_position: glm::vec2(0.0, 0.0),
            mouse_state: ElementState::Released,
            previout_mouse_state: ElementState::Released,
            mouse_area_stack: vec![],
            previous_time: now,
            current_time: now,
        }
    }

    pub fn get_screen_bbox(&self) -> RenderBox {
        get_screen_bbox(self.window_size)
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

    pub fn update_window_size(&mut self, window_size: glm::Vec2) {
        self.window_size = window_size;
    }

    pub fn update_mouse_position(&mut self, screen_position: glm::Vec2) {
        self.mouse_position = screen_position_to_ndc(screen_position, self.window_size);
    }

    pub fn update_mouse_state(&mut self, state: ElementState) {
        self.mouse_state = state;
    }

    pub fn get_mouse_position(&self) -> glm::Vec2 {
        self.mouse_position
    }

    pub fn get_previous_mouse_position(&self) -> glm::Vec2 {
        self.previous_mouse_position
    }

    pub fn get_mouse_position_delta(&self) -> glm::Vec2 {
        self.mouse_position - self.previous_mouse_position
    }

    pub fn get_mouse_state(&self) -> ElementState {
        self.mouse_state
    }

    pub fn just_pressed(&self) -> bool {
        self.mouse_state == ElementState::Pressed && self.previout_mouse_state == ElementState::Released
    }

    pub fn just_released(&self) -> bool {
        self.mouse_state == ElementState::Released && self.previout_mouse_state == ElementState::Pressed
    }

    pub fn new_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn push_area(&mut self, area: ListeningArea) {
        self.areas.push(area);
    }

    pub fn on_frame_end(&mut self, window: &Window) {
        let (area, cursor) = resolve_conflicts(&mut self.areas, self.mouse_position);
        self.mouse_area_stack = area;
        window.set_cursor(cursor);
        self.areas.clear();

        self.previous_mouse_position = self.mouse_position;
        self.previout_mouse_state = self.mouse_state;

        self.previous_time = self.current_time;
        self.current_time = Instant::now();
    }

    pub fn get_delta(&self) -> f32 {
        self.current_time.duration_since(self.previous_time).as_secs_f32().min(0.1)
    }

    pub fn get_top_area(&self) -> Option<u64> {
        self.mouse_area_stack.last().copied()
    }
}