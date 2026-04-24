use crate::{MainContext, game::{logic::{GameState, world::World}, render::{robot_info::RobotInfo, world::ViewWorld}}, graphics::renderer::Renderer};
use nalgebra_glm as glm;

pub const UI_BASE_DEPTH: u16 = 1000;

pub mod display_module;
pub mod robot;
pub mod world;
pub mod robot_info;

pub struct RenderState<'a> {
    world: ViewWorld<'a>,
    robot_info: RobotInfo,
}

impl<'a> RenderState<'a> {
    pub fn new(renderer: &'a Renderer, logic_world: &World) -> Self {
        Self {
            world: ViewWorld::new(renderer, logic_world),
            robot_info: Default::default(),
        }
    }

    pub fn send(&mut self, logic_state: &GameState, context: &mut MainContext) {
        for robot in logic_state.get_robots().iter() {
            robot.send(context);
        }

        self.world.send(logic_state.get_world());

        self.robot_info.update(logic_state.robot_info_opened, context.ui);
        self.robot_info.send(context.simple_shape_renderer, context.ui);
    }

    pub fn draw(&self, proj_matrix: &glm::Mat4) {
        self.world.draw(proj_matrix);
    }
}