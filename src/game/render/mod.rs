use crate::{MainContext, game::{logic::{GameState, fabricator::Fabricator, robot::Robot, world::World}, render::{robot::ROBOT_DEPTH, robot_info::{EntityId, RobotInfo, RobotInfoContext}, world::ViewWorld}}, graphics::renderer::{Renderer, positioning::{PositionMode, RenderBox}}, ui::{events::ActionType, simple_area::SimpleArea, simple_area_manager::AreaId, ui_element::UiElement}};
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
pub mod memory_visualizer;
pub mod fabricator;

pub struct RenderState<'a> {
    world: ViewWorld<'a>,
    robot_info: Option<RobotInfo>,
    buttons: Vec<(EntityId, AreaId)>,
}

impl<'a> RenderState<'a> {
    pub fn new(renderer: &'a Renderer, logic_world: &World) -> Self {
        Self {
            world: ViewWorld::new(renderer, logic_world),
            robot_info: None,
            buttons: vec![],
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
                for (_, (entity, button)) in self.buttons.iter().enumerate() {
                    if
                        id == *button &&
                        self.robot_info.as_ref().map(|r| r.id) != Some(*entity)
                    {
                        let instructions = match entity {
                            EntityId::Robot(id) => logic_state.get_robots()[*id as usize].get_computer().instructions.get(),
                            EntityId::Fabricator(idx) => logic_state.fabricators[*idx as usize].get_code(),
                        };
                        self.robot_info = Some(RobotInfo::new(
                            *entity,
                            *instructions,
                        ));
                    }
                }
            },
            _ => {},
        }

        if let Some(robot_info) = &mut self.robot_info {
            match robot_info.id {
                EntityId::Robot(id) => {
                    if let Some(robot) = logic_state.get_robots().get(id as usize) {
                        let context = &mut RobotInfoContext {
                            mc: context,
                            data: Some(robot.computer.registers.get()),
                        };

                        let ndc_bbox = context.mc.ui.get_ndc_render_box();
                        robot_info.update(&Self::get_robot_info_bbox(&ndc_bbox), UI_BASE_DEPTH, context);
                        if let Some(code) = robot_info.finished() {
                            logic_state.write_code(id as usize, code);
                            self.robot_info = None;
                        }
                    }
                },
                EntityId::Fabricator(idx) => {
                    let context = &mut RobotInfoContext {
                        mc: context,
                        data: None,
                    };

                    let ndc_bbox = context.mc.ui.get_ndc_render_box();
                    robot_info.update(&Self::get_robot_info_bbox(&ndc_bbox), UI_BASE_DEPTH, context);
                    if
                        let Some(code) = robot_info.finished() &&
                        let Some(fabricator) = logic_state.fabricators.get_mut(idx as usize)
                    {
                        fabricator.start_code = code;
                        self.robot_info = None;
                    }
                },
            }
        }

        self.buttons.clear();
        for (i, robot) in logic_state.get_robots().iter_enumerate_unordered() {
            self.buttons.push((EntityId::Robot(i as u32), context.ui.simple_area_manager.push(SimpleArea::new(
                Robot::get_render_box(robot.movement_module.get_position()).world_to_ndc_pos(&context.ui.camera),
                ROBOT_DEPTH,
            ).with_cursor(CursorIcon::Pointer))));
        }

        for (i, fabricator) in logic_state.fabricators.iter().enumerate() {
            self.buttons.push((EntityId::Fabricator(i as u32), context.ui.simple_area_manager.push(SimpleArea::new(
                Fabricator::get_bbox(fabricator.position.map(|x| x as f32)).world_to_ndc_pos(&context.ui.camera),
                ROBOT_DEPTH,
            ).with_cursor(CursorIcon::Pointer))));
        }
    }

    pub fn send(&mut self, logic_state: &GameState, context: &mut MainContext) {
        if let Some(robot_info) = &mut self.robot_info {
            match robot_info.id {
                EntityId::Robot(id) => {
                    if let Some(robot) = logic_state.get_robots().get(id as usize) {
                        let context = &mut RobotInfoContext {
                            mc: context,
                            data: Some(robot.computer.registers.get()),
                        };
                        robot_info.send(context);
                    }
                },
                EntityId::Fabricator(_) => {
                    let context = &mut RobotInfoContext {
                        mc: context,
                        data: None,
                    };
                    robot_info.send(context);
                }
            }
        }

        for robot in logic_state.get_robots().iter_unordered() {
            robot.send(context);
        }

        for fabricator in logic_state.fabricators.iter() {
            fabricator.send(context);
        }

        self.world.send(logic_state.get_world());
    }

    pub fn draw(&self, proj_matrix: &glm::Mat4) {
        self.world.draw(proj_matrix);
    }
}
