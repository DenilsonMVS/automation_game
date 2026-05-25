use crate::{MainContext, game::{logic::{GameState, fabricator::Fabricator, robot::Robot}, render::{robot::ROBOT_DEPTH, robot_info::{EntityId, RobotInfo, RobotInfoContext}, world::ViewWorld}}, graphics::renderer::{Renderer, positioning::{PositionMode, RenderBox}}, ui::{events::ActionType, simple_area::SimpleArea, simple_area_manager::AreaId, ui_element::UiElement}};
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
pub mod targets;

pub struct RenderState<'a> {
    world: ViewWorld<'a>,
    robot_info: Option<RobotInfo>,
    buttons: Vec<(EntityId, AreaId)>,
}

impl<'a> RenderState<'a> {
    pub fn new(renderer: &'a Renderer, logic_state: &GameState) -> Self {
        Self {
            world: ViewWorld::new(renderer, logic_state),
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

    pub fn update(&mut self, logic_state: &mut GameState, context: &mut MainContext, update_buttons: bool) {
        match context.ui.events.mouse_actions {
            ActionType::NonePress => self.robot_info = None,
            ActionType::AreaPress(id, _) => {
                for (_, (entity, button)) in self.buttons.iter().enumerate() {
                    if
                        id == *button &&
                        self.robot_info.as_ref().map(|r| r.id) != Some(*entity)
                    {
                        let instructions = match entity {
                            EntityId::Robot(chunk_id, id) => logic_state.get_robot(*chunk_id, *id).map(|robot| robot.computer.instructions.get()),
                            EntityId::Fabricator(chunk_id, idx) => logic_state.get_fabricator(*chunk_id, *idx).map(|fabricator| fabricator.get_code()),
                        };

                        if let Some(instructions) = instructions {
                            self.robot_info = Some(RobotInfo::new(
                                *entity,
                                *instructions,
                            ));
                        }
                    }
                }
            },
            _ => {},
        }

        if let Some(robot_info) = &mut self.robot_info {
            match robot_info.id {
                EntityId::Robot(chunk_id, id) => {
                    if let Some(robot) = logic_state.get_robot(chunk_id, id) {
                        let context = &mut RobotInfoContext {
                            mc: context,
                            data: Some(robot.computer.registers.get()),
                        };

                        let ndc_bbox = context.mc.ui.get_ndc_render_box();
                        robot_info.update(&Self::get_robot_info_bbox(&ndc_bbox), UI_BASE_DEPTH, context);
                        if let Some(code) = robot_info.finished() {
                            logic_state.write_code(chunk_id, id, code);
                            self.robot_info = None;
                        }
                    }
                },
                EntityId::Fabricator(chunk_id, id) => {
                    let context = &mut RobotInfoContext {
                        mc: context,
                        data: None,
                    };

                    let ndc_bbox = context.mc.ui.get_ndc_render_box();
                    robot_info.update(&Self::get_robot_info_bbox(&ndc_bbox), UI_BASE_DEPTH, context);
                    if
                        let Some(code) = robot_info.finished() &&
                        let Some(fabricator) = logic_state.get_fabricator_mut(chunk_id, id)
                    {
                        fabricator.start_code = code;
                        self.robot_info = None;
                    }
                },
            }
        }

        self.buttons.clear();
        if !update_buttons {
            return;
        }
        
        for (chunk_id, chunk) in logic_state.chunks.iter() {
            for (i, robot) in chunk.robots.iter_enumerate_unordered() {
                self.buttons.push((
                    EntityId::Robot(*chunk_id, i as u32),
                    context.ui.simple_area_manager.push(SimpleArea::new(
                        Robot::get_render_box(robot.movement_module.get_position()).world_to_ndc_pos(&context.ui.camera),
                        ROBOT_DEPTH,
                    ).with_cursor(CursorIcon::Pointer))
                ));
            }

            for (i, fabricator) in chunk.fabricators.iter().enumerate() {
                self.buttons.push((
                    EntityId::Robot(*chunk_id, i as u32),
                    context.ui.simple_area_manager.push(SimpleArea::new(
                        Fabricator::get_bbox(fabricator.position.map(|x| x as f32)).world_to_ndc_pos(&context.ui.camera),
                        ROBOT_DEPTH,
                    ).with_cursor(CursorIcon::Pointer))
                ));
            }
        }
    }

    pub fn send(&mut self, logic_state: &GameState, context: &mut MainContext) {
        if let Some(robot_info) = &mut self.robot_info {
            match robot_info.id {
                EntityId::Robot(chunk_id, id) => {
                    if let Some(robot) = logic_state.get_robot(chunk_id, id) {
                        let context = &mut RobotInfoContext {
                            mc: context,
                            data: Some(robot.computer.registers.get()),
                        };
                        robot_info.send(context);
                    }
                },
                EntityId::Fabricator(..) => {
                    let context = &mut RobotInfoContext {
                        mc: context,
                        data: None,
                    };
                    robot_info.send(context);
                }
            }
        }

        for (chunk_id, chunk) in logic_state.chunks.iter() {
            chunk.targets.send(*chunk_id, context);

            for robot in chunk.robots.iter_unordered() {
                robot.send(context);
            }

            for fabricator in chunk.fabricators.iter() {
                fabricator.send(context);
            }
        }

        self.world.send(logic_state);
    }

    pub fn draw(&self, proj_matrix: &glm::Mat4) {
        self.world.draw(proj_matrix);
    }
}
