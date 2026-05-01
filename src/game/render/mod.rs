use crate::{MainContext, game::{logic::{GameState, robot::Robot, world::World}, render::{robot::ROBOT_DEPTH, robot_info::{RobotInfo, RobotInfoEventContext}, world::ViewWorld}, scene::{EventReceiver, EventType, UiElement}}, graphics::renderer::{Renderer, fonts::Font, simple_shape_renderer::SimpleShapeRenderer}, ui::{Ui, simple_area::SimpleArea}};
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
    robot_buttons: Vec<usize>,
}

impl<'a> RenderState<'a> {
    pub fn new(renderer: &'a Renderer, logic_world: &World) -> Self {
        Self {
            world: ViewWorld::new(renderer, logic_world),
            robot_info: None,
            robot_buttons: vec![],
        }
    }

    pub fn update(&mut self, logic_state: &GameState, context: &mut MainContext) {
        for (i, robot) in logic_state.get_robots().iter().enumerate() {
            let updated_simple_area = SimpleArea::new_button(
                Robot::get_render_box(robot.get_position()).world_to_ndc_pos(&context.ui.camera),
                ROBOT_DEPTH
            );
            if i >= self.robot_buttons.len() {
                self.robot_buttons.push(context.ui.simple_area_manager.push_simple_area(updated_simple_area));
            } else {
                context.ui.simple_area_manager.update_simple_area(
                    self.robot_buttons[i],
                    updated_simple_area,
                );
            }
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
    pub robots: &'a [Robot],
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

        let idx = self.robot_buttons
            .iter()
            .enumerate()
            .find(|(_, button_id)| **button_id == id)
            .map(|(i, _)| i);

        if let Some(idx) = idx && let Some(robot) = context.robots.get(idx) {
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
