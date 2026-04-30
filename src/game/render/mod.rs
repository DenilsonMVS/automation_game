use crate::{MainContext, game::{logic::{GameState, robot::Robot, world::World}, render::{robot_info::RobotInfo, world::ViewWorld}, scene::EventType}, graphics::renderer::{Renderer, fonts::Font}, ui::Ui};
use nalgebra_glm as glm;

pub const UI_BASE_DEPTH: u16 = 1000;

pub mod display_module;
pub mod robot;
pub mod world;
pub mod robot_info;
pub mod dnd;
pub mod instruction_unit;
pub mod menu;

pub struct RenderState<'a> {
    world: ViewWorld<'a>,
    robot_info: Option<RobotInfo>,
}

impl<'a> RenderState<'a> {
    pub fn new(renderer: &'a Renderer, logic_world: &World) -> Self {
        Self {
            world: ViewWorld::new(renderer, logic_world),
            robot_info: None,
        }
    }

    pub fn receive_move_event(&mut self, id: usize, offset: glm::Vec2) {
        if let Some(robot_info) = &mut self.robot_info {
            robot_info.receive_move_event(id, offset);
        }
    }

    pub fn receive_hover_event(&mut self, event: &EventType) {
        if let Some(robot_info) = &mut self.robot_info {
            robot_info.receive_hover_event(event);
        }
    }

    pub fn receive_button_click(&mut self, button_id: usize, ui: &mut Ui, font: &Font, robot: Option<&Robot>) {
        if let Some(robot_info) = &mut self.robot_info {
            robot_info.receive_button_click(button_id, ui, font);
        }

        let robot_info_button_id = self.robot_info.as_ref().map(|r| r.robot_button_id);
        if robot_info_button_id == Some(button_id) {
            return;
        }

        if let Some(robot) = robot {
            self.robot_info = Some(RobotInfo::new(ui, button_id, robot));
        }
    }

    pub fn receive_release_event(&mut self, id: usize) {
        if let Some(robot_info) = &mut self.robot_info {
            robot_info.receive_release_event(id);
        }
    }

    pub fn receive_click_outside(&mut self) {
        self.robot_info = None;
    }

    pub fn send(&mut self, logic_state: &GameState, context: &mut MainContext) {
        for robot in logic_state.get_robots().iter() {
            robot.send(context);
        }

        self.world.send(logic_state.get_world());

        if let Some(robot_info) = &mut self.robot_info {            
            robot_info.send(context);
        }
    }

    pub fn draw(&self, proj_matrix: &glm::Mat4) {
        self.world.draw(proj_matrix);
    }
}