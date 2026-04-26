use crate::{MainContext, game::{logic::{GameState, robot::Robot, world::World}, render::{robot_info::RobotInfo, world::ViewWorld}}, graphics::renderer::Renderer, ui::Ui};
use nalgebra_glm as glm;

pub const UI_BASE_DEPTH: u16 = 1000;

pub mod display_module;
pub mod robot;
pub mod world;
pub mod robot_info;

pub struct RenderState<'a> {
    world: ViewWorld<'a>,
    robot_info: Option<RobotInfo>,
    received_click: bool,
    clicked_some_ui_area: bool,
}

impl<'a> RenderState<'a> {
    pub fn new(renderer: &'a Renderer, logic_world: &World) -> Self {
        Self {
            world: ViewWorld::new(renderer, logic_world),
            robot_info: None,
            received_click: false,
            clicked_some_ui_area: false,
        }
    }

    pub fn receive_generic_click(&mut self) {
        self.received_click = true;
    }

    pub fn receive_ui_click(&mut self) {
        self.clicked_some_ui_area = true;
    }

    pub fn receive_move_event(&mut self, id: usize, offset: glm::Vec2) {
        if let Some(robot_info) = &mut self.robot_info {
            robot_info.receive_move_event(id, offset);
        }
    }

    pub fn receive_button_click(&mut self, button_id: usize, ui: &mut Ui, robot: &Robot) {
        let robot_info_button_id = self.robot_info.as_ref().map(|r| r.robot_button_id);
        if robot_info_button_id == Some(button_id) {
            return;
        }

        self.robot_info = Some(RobotInfo::new(ui, button_id, robot));
    }

    pub fn receive_release_event(&mut self, id: usize) {
        if let Some(robot_info) = &mut self.robot_info {
            robot_info.receive_release_event(id);
        }
    }

    pub fn receive_finish_event(&mut self) {
        if self.received_click && !self.clicked_some_ui_area {
            self.robot_info = None;
        }

        self.received_click = false;
        self.clicked_some_ui_area = false;
    }

    pub fn send(&mut self, logic_state: &GameState, context: &mut MainContext) {
        for robot in logic_state.get_robots().iter() {
            robot.send(context);
        }

        self.world.send(logic_state.get_world());

        if
            let Some(robot_info) = &self.robot_info &&
            let Some(robot) = logic_state.get_robots().iter().find(|robot| robot.button_id == robot_info.robot_button_id)
        {            
            robot_info.send(context, robot);
        }
    }

    pub fn draw(&self, proj_matrix: &glm::Mat4) {
        self.world.draw(proj_matrix);
    }
}