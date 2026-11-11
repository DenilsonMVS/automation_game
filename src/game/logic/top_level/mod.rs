
pub mod compute_context;
pub mod computer;
pub mod robot;
pub mod memory;
pub mod instruction;
pub mod display_module;
pub mod chunk;
mod world;
mod mining_module;

use std::sync::mpsc;

use nalgebra_glm as glm;

use crate::{game::{logic::{LogicMessage, SimulationState, physics::PhysicsSnapshot, top_level::{chunk::Chunk, display_module::DisplayModule, robot::{RobotJoin, RobotLogic}, world::World}}, render::RenderMessage}, open_simplex::OpenSimplex, snapshot_manager::Consumer};

#[derive(Debug, Clone)]
pub struct TopLevelSnapshot {
    pub robot_targets: Vec<glm::Vec2>,
    pub robot_display_values: Vec<Option<u8>>,
    pub watched_robots: Vec<RobotLogic>,
}

impl TopLevelSnapshot {
    pub fn new() -> Self {
        Self {
            robot_targets: vec![],
            robot_display_values: vec![],
            watched_robots: vec![],
        }
    }
}

pub struct TopLevelState {
    // open_simplex: OpenSimplex,
    world: World,
    physics: Consumer<PhysicsSnapshot>,
    targets: Vec<glm::Vec2>,
    display_modules: Vec<DisplayModule>,
    robots: Vec<RobotLogic>,
    render_tx: mpsc::Sender<RenderMessage>,
}

impl TopLevelState {
    pub(super) fn new(physics: Consumer<PhysicsSnapshot>, render_tx: mpsc::Sender<RenderMessage>) -> Self {
        let open_simplex = OpenSimplex::new(0);
        let chunk = Chunk::new(Chunk::proc_generated(glm::vec2(0, 0), &open_simplex));
        let mut world = World::new();
        world.add_chunk(glm::vec2(0, 0), chunk, &render_tx);
        
        Self {
            world,
            // open_simplex,
            physics,
            targets: vec![],
            robots: vec![],
            display_modules: vec![],
            render_tx,
        }
    }

    fn push_robot(&mut self, robot: RobotLogic, position: glm::Vec2) {
        self.targets.push(position);
        self.robots.push(robot);
        self.display_modules.push(DisplayModule::new());
    }

    fn get_snapshot(&self, watched_robots: Vec<RobotLogic>) -> TopLevelSnapshot {
        TopLevelSnapshot {
            robot_targets: self.targets.clone(),
            robot_display_values: self.display_modules
                .iter()
                .map(|v| v.get_value())
                .collect(),
            watched_robots,
        }
    }

    fn handle_messages(&mut self, rx: &mpsc::Receiver<LogicMessage>) {
        for message in rx.try_iter() {
            match message {
                LogicMessage::PushRobot(instructions, position) =>
                    self.push_robot(RobotLogic::new(instructions), position),
                LogicMessage::StartRobotWatch(idx) => {
                    if let Some(robot) = self.robots.get_mut(idx) {
                        robot.watched = true;
                    }
                },
                LogicMessage::StopRobotWatch(idx) => {
                    if let Some(robot) = self.robots.get_mut(idx) {
                        robot.watched = false;
                    }
                },
            }
        }
    }
}

impl SimulationState for TopLevelState {
    type Snapshot = TopLevelSnapshot;
    type Message = LogicMessage;
    fn update(&mut self, delta: f32, rx: &mpsc::Receiver<Self::Message>) -> Self::Snapshot {
        self.handle_messages(rx);

        let updates = self.world.take_updates();
        if updates.len() > 0 {
            self.render_tx.send(RenderMessage::ModifyChunk(updates)).unwrap();
        }

        let mut watched_robots = vec![];
        let physics = self.physics.acquire();
        for (((robot, target), physics), display_module) in
            self.robots.iter_mut()
                .zip(self.targets.iter_mut())
                .zip(physics.robots.iter())
                .zip(self.display_modules.iter_mut())
        {
            let mut join = RobotJoin::new(robot, display_module, target, physics);
            join.update(delta, &mut self.world);

            if robot.watched {
                watched_robots.push(robot.clone());
            }
        }

        self.get_snapshot(watched_robots)
    }
}