use crate::{MainContext, game::{logic::{GameState, robot::Robot, world::World}, render::{robot::ROBOT_DEPTH, world::ViewWorld}}, graphics::renderer::{Renderer}, ui::simple_area::SimpleArea};
use nalgebra_glm as glm;
use winit::window::CursorIcon;

pub const UI_BASE_DEPTH: u16 = 1000;

pub mod display_module;
pub mod robot;
pub mod world;

pub struct RenderState<'a> {
    world: ViewWorld<'a>,
    robot_buttons: Vec<u32>,
}

impl<'a> RenderState<'a> {
    pub fn new(renderer: &'a Renderer, logic_world: &World) -> Self {
        Self {
            world: ViewWorld::new(renderer, logic_world),
            // robot_info: None,
            robot_buttons: vec![],
        }
    }

    pub fn update(&mut self, logic_state: &GameState, context: &mut MainContext) {
        self.robot_buttons.clear();
        for robot in logic_state.get_robots().iter() {
            self.robot_buttons.push(context.ui.simple_area_manager.push(SimpleArea::new(
                Robot::get_render_box(robot.get_position()).world_to_ndc_pos(&context.ui.camera),
                ROBOT_DEPTH,
            ).with_cursor(CursorIcon::Pointer)));
        }
    }

    pub fn send(&mut self, logic_state: &GameState, context: &mut MainContext) {
        for robot in logic_state.get_robots().iter() {
            robot.send(context);
        }

        self.world.send(logic_state.get_world());
    }

    pub fn draw(&self, proj_matrix: &glm::Mat4) {
        self.world.draw(proj_matrix);
    }
}
