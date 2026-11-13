use std::sync::mpsc;

use nalgebra_glm as glm;
use winit::window::CursorIcon;
use crate::{game::{logic::{LogicMessage, physics::PhysicsSnapshot, top_level::{self, TopLevelSnapshot, chunk::GridPosition, }}, render::{robot::{Robot, send_robot}, robot_info::RobotInfo, world::World}}, graphics::renderer::{Renderer, fonts::font::Font}, ui::{ListeningArea, Ui}};

pub mod chunk;
pub mod robot;
pub mod world;
pub mod robot_info;


#[derive(Debug, Clone)]
pub enum RenderMessage {
    ChunkGenerated((glm::IVec2, Box<top_level::chunk::Chunk>)),
    ModifyChunk(Vec<(glm::IVec2, GridPosition)>),
}

pub struct RenderState {
    world: World,
    robot_buttons: Vec<u64>,
    robot_info: Option<RobotInfo>,
    logic_tx: mpsc::Sender<LogicMessage>,
}

impl RenderState {
    pub fn new(logic_tx: mpsc::Sender<LogicMessage>) -> Self {
        Self {
            world: World::new(),
            robot_buttons: vec![],
            robot_info: None,
            logic_tx,
        }
    }

    fn handle_messages(&mut self, rx: &mpsc::Receiver<RenderMessage>) {
        for message in rx.try_iter() {
            match message {
                RenderMessage::ChunkGenerated((position, chunk)) =>
                    self.world.add_chunk(position, &chunk),
                RenderMessage::ModifyChunk(modifications) =>
                    self.world.modify_chunk(&modifications),
            }
        }
    }

    pub fn update(
        &mut self,
        ui: &mut Ui,
        physics: &PhysicsSnapshot,
        logic: &TopLevelSnapshot,
        renderer: &mut Renderer,
        font: &mut Font,
        rx: &mpsc::Receiver<RenderMessage>,
    ) {
        self.handle_messages(rx);

        let camera = ui.camera;

        let top_area = ui.get_top_area();
        let mut hovering_robot_idx = None;

        for (i, robot) in physics.robots.iter().enumerate() {
            let id = if let Some(&existing_id) = self.robot_buttons.get(i) {
                existing_id
            } else {
                let new_id = ui.new_id();
                self.robot_buttons.push(new_id);
                new_id
            };

            if Some(id) == top_area {
                hovering_robot_idx = Some(i);
            }

            let bbox = camera.world_to_ndc_box(&robot::get_bbox(robot.position));
            ui.push_area(ListeningArea {
                bbox, id,
                depth: 64000,
                cursor: CursorIcon::Pointer,
            });
        }

        if ui.just_released() {
            if let Some(idx) = hovering_robot_idx {
                if let Some(cur_info) = &self.robot_info {
                    let cur_idx = cur_info.get_robot_idx();
                    if cur_idx != idx {
                        self.robot_info = Some(RobotInfo::new(idx, ui));
                        self.logic_tx.send(LogicMessage::StartRobotWatch(idx)).unwrap();
                        self.logic_tx.send(LogicMessage::StopRobotWatch(cur_idx)).unwrap();
                    }
                } else {
                    self.robot_info = Some(RobotInfo::new(idx, ui));
                    self.logic_tx.send(LogicMessage::StartRobotWatch(idx)).unwrap();
                }
            } else if top_area.is_none() {
                if let Some(cur_info) = &self.robot_info {
                    self.logic_tx.send(LogicMessage::StopRobotWatch(cur_info.get_robot_idx())).unwrap();
                }
                self.robot_info = None;
            }
        }
        
        if let Some(robot_info) = &mut self.robot_info {
            robot_info.update(ui, renderer, font, &logic.watched_robots);
        }

        let extra_time = physics.finish_time.elapsed().as_secs_f32();
        for (physics, display) in
            physics.robots.iter()
            .zip(logic.robot_display_values.iter())
        {
            let robot = Robot {
                position: physics.position,
                speed: physics.speed,
                display: *display,
            };
            send_robot(&robot, renderer, font, extra_time);
        }
    }

    pub fn draw(&self, u_vp: &glm::Mat4, renderer: &mut Renderer) {
        self.world.draw(renderer, u_vp);
    }
}