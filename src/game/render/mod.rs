use crate::{MainContext, game::{logic::{GameState, robot::Robot, world::World}, render::{robot_info::{RobotInfo, RobotInfoEventContext}, world::ViewWorld}, scene::{EventReceiver, EventType, UiElement}}, graphics::renderer::{Renderer, fonts::Font, simple_shape_renderer::SimpleShapeRenderer}, ui::Ui};
use nalgebra_glm as glm;

pub const UI_BASE_DEPTH: u16 = 1000;

pub mod display_module;
pub mod robot;
pub mod world;
pub mod robot_info;
pub mod dnd;
pub mod instruction_unit;
pub mod menu;
pub mod rendered_area;
pub mod rendered_button;

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

    pub fn send(&mut self, logic_state: &GameState, context: &mut MainContext) {
        for robot in logic_state.get_robots().iter() {
            robot.send(context);
        }

        self.world.send(logic_state.get_world());

        if let Some(robot_info) = &mut self.robot_info {            
            robot_info.update(context.ui.get_ndc_render_box().get_top_left(), UI_BASE_DEPTH, context);
            robot_info.send(context);
        }
    }

    pub fn draw(&self, proj_matrix: &glm::Mat4) {
        self.world.draw(proj_matrix);
    }
}

pub struct RenderEventContext<'a, 'b> {
    pub ui: &'a mut Ui,
    pub font: &'a mut Font<'b>,
    pub simple_shape_renderer: &'a mut SimpleShapeRenderer<'b>,
    pub robot: Option<&'a Robot>,
}

impl<'a, 'b, 'c> EventReceiver<RenderEventContext<'a, 'b>> for RenderState<'c> {
    fn on_click_outside(&mut self, _context: &mut RenderEventContext<'a, 'b>) {
        self.robot_info = None;
    }

    fn on_simple_area_pressed(&mut self, id: usize, context: &mut RenderEventContext<'a, 'b>) {
        let robot_info_button_id = self.robot_info.as_ref().map(|r| r.robot_button_id);
        if robot_info_button_id == Some(id) {
            return;
        }

        if let Some(robot) = context.robot {
            self.robot_info = Some(RobotInfo::new(context.ui, id, robot));
        }
    }

    fn propagate_children(&mut self, event: &EventType, context: &mut RenderEventContext<'a, 'b>) {
        let mut robot_info_context = RobotInfoEventContext {
            ui: context.ui,
            font: context.font,
        };

        if let Some(robot_info) = &mut self.robot_info {
            robot_info.on_event(event, &mut robot_info_context);
        }
    }
}
