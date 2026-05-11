use crate::{MainContext, game::{logic::{GameState, robot::Robot, world::World}, render::{robot::ROBOT_DEPTH, robot_info::RobotInfo, world::ViewWorld}}, graphics::renderer::{Renderer, positioning::{PositionMode, RenderBox}}, ui::{events::ActionType, simple_area::SimpleArea, simple_area_manager::AreaId, ui_element::UiElement}};
use nalgebra_glm as glm;
use winit::window::CursorIcon;

pub const UI_BASE_DEPTH: u16 = 1000;

pub const INFO_WIDTH: f32 = 1.2;
pub const INFO_HEIGHT: f32 = 1.6;

pub mod display_module;
pub mod robot;
pub mod world;
pub mod robot_info;
pub mod instruction_editor;
pub mod instruction_view;
pub mod colored_text;
pub mod go_back_arrow;

pub struct RenderState<'a> {
    world: ViewWorld<'a>,
    robot_info: Option<RobotInfo>,
    robot_buttons: Vec<(u32, AreaId)>,
}

impl<'a> RenderState<'a> {
    pub fn new(renderer: &'a Renderer, logic_world: &World) -> Self {
        Self {
            world: ViewWorld::new(renderer, logic_world),
            robot_info: None,
            robot_buttons: vec![],
        }
    }

    fn get_robot_info_bbox(ndc_bbox: &RenderBox) -> RenderBox {
        const MARGIN: f32 = 0.03;
        RenderBox::new_simple(
            ndc_bbox.get_top_left() + glm::vec2(MARGIN, -MARGIN),
            PositionMode::TopLeft,
            glm::vec2(INFO_WIDTH, INFO_HEIGHT),
        )
    }

    pub fn update(&mut self, logic_state: &mut GameState, context: &mut MainContext) {
        match context.ui.events.mouse_actions {
            ActionType::NonePress => self.robot_info = None,
            ActionType::AreaPress(id, _) => {
                for (_, (robot_id, button)) in self.robot_buttons.iter().enumerate() {
                    if
                        id == *button &&
                        self.robot_info.as_ref().map(|r| r.robot_id) != Some(*robot_id)
                    {
                        self.robot_info = Some(RobotInfo::new(
                            *robot_id,
                            *logic_state.get_robots()[*robot_id as usize].get_computer().instructions.get(),
                        ));
                    }
                }
            },
            _ => {},
        }

        if let Some(robot_info) = &mut self.robot_info {
            let ndc_bbox = context.ui.get_ndc_render_box();
            robot_info.update(&Self::get_robot_info_bbox(&ndc_bbox), UI_BASE_DEPTH, context);
            if let Some(code) = robot_info.finished() {
                logic_state.write_code(robot_info.robot_id as usize, code);
                self.robot_info = None;
            }
        }

        self.robot_buttons.clear();
        for (i, robot) in logic_state.get_robots().iter_enumerate_unordered() {
            self.robot_buttons.push((i as u32, context.ui.simple_area_manager.push(SimpleArea::new(
                Robot::get_render_box(robot.get_position()).world_to_ndc_pos(&context.ui.camera),
                ROBOT_DEPTH,
            ).with_cursor(CursorIcon::Pointer))));
        }
    }

    pub fn send(&mut self, logic_state: &GameState, context: &mut MainContext) {
        if let Some(robot_info) = &mut self.robot_info {
            robot_info.send(context);
        }

        for robot in logic_state.get_robots().iter_unordered() {
            robot.send(context);
        }

        self.world.send(logic_state.get_world());
    }

    pub fn draw(&self, proj_matrix: &glm::Mat4) {
        self.world.draw(proj_matrix);
    }
}
