

use std::{sync::mpsc, time::Instant};

use bitvec::BitArr;
use nalgebra_glm as glm;

use crate::{game::logic::{PhysicsMessage, SimulationState, top_level::{TopLevelSnapshot, chunk::CHUNK_SIZE}}, snapshot_manager::Consumer};

#[derive(Debug, Clone, Copy)]
pub struct RobotPhysics {
    pub position: glm::Vec2,
    pub speed: glm::Vec2,
}

impl RobotPhysics {
    fn calculate_accelleration(&self, target: glm::Vec2) -> glm::Vec2 {
        const PROPORTIONAL: f32 = 10.0;
        const DERIVATIVE: f32 = -6.0;

        let distance = target - self.position;
        let d = self.speed * DERIVATIVE;
        let p = distance * PROPORTIONAL;
        return p + d;
    }

    pub fn update(&mut self, delta: f32, target: glm::Vec2) {
        const EPS: f32 = 0.0001;
        const MAX_ACCELLERATION: f32 = 1.0;
        const DRAG: f32 = 1.0;

        let sq = |x| x * x;

        let acc = self.calculate_accelleration(target);
        let acc_mag = glm::length(&acc);
        let acc_adjust = acc_mag.min(MAX_ACCELLERATION);
        let acc = acc / (acc_mag + EPS) * acc_adjust;

        let speed_mag = glm::length(&self.speed);
        let norm_speed = self.speed / (speed_mag + EPS);
        let drag_vector = -norm_speed * sq(speed_mag) * DRAG;

        let all_acc = acc + drag_vector;

        self.position += self.speed * delta + all_acc * sq(delta) * 0.5;
        self.speed += all_acc * delta;
    }
}

#[derive(Debug)]
pub struct PhysicsState {
    logic: Consumer<TopLevelSnapshot>,
    robots: Vec<RobotPhysics>,
    finish_time: Instant,
    chunk_obstacles: [BitArr!(for CHUNK_SIZE); CHUNK_SIZE],
}

impl PhysicsState {
    pub(super) fn new(logic: Consumer<TopLevelSnapshot>) -> Self {
        Self {
            robots: vec![],
            finish_time: Instant::now(),
            logic,
            chunk_obstacles: [Default::default(); _],
        }
    }

    fn get_snapshot(&self) -> PhysicsSnapshot {
        PhysicsSnapshot {
            robots: self.robots.clone(),
            finish_time: self.finish_time,
        }
    }

    fn modify_chunk(&mut self, position: glm::IVec2, value: bool) {
        let chunk_id = position.map(|x| x.div_euclid(CHUNK_SIZE as i32));
        let chunk_off = position.map(|x| x.rem_euclid(CHUNK_SIZE as i32));

        if chunk_id == glm::vec2(0, 0) {
            self.chunk_obstacles[chunk_off.y as usize].set(chunk_off.x as usize, value);
        }
    }

    fn handle_messages(&mut self, rx: &mpsc::Receiver<PhysicsMessage>) {
        for message in rx.try_iter() {
            match message {
                PhysicsMessage::PushRobot(physics) =>
                    self.robots.push(physics),
                PhysicsMessage::ModifyChunk(changes) => {
                    for (position, value) in changes.iter() {
                        self.modify_chunk(*position, *value);
                    }
                },
            }
        }
    }
}

impl SimulationState for PhysicsState {
    type Snapshot = PhysicsSnapshot;
    type Message = PhysicsMessage;
    fn update(&mut self, delta: f32, rx: &mpsc::Receiver<Self::Message>) -> Self::Snapshot {
        self.handle_messages(rx);

        let logic = self.logic.acquire();
        for (robot, target) in self.robots.iter_mut().zip(logic.robot_targets.iter()) {
            robot.update(delta, *target);
        }
        self.finish_time = Instant::now();

        self.get_snapshot()
    }
}

#[derive(Debug, Clone)]
pub struct PhysicsSnapshot {
    pub robots: Vec<RobotPhysics>,
    pub finish_time: Instant,
}

impl PhysicsSnapshot {
    pub fn new() -> Self {
        Self {
            robots: vec![],
            finish_time: Instant::now(),
        }
    }
}
