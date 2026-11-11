
pub mod top_level;
pub mod physics;

const INSTRUCTION_PERIOD: f32 = 0.25;

use std::{sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc}, thread, time::{Duration, Instant}};
use crate::{game::{logic::{physics::{PhysicsSnapshot, PhysicsState, RobotPhysics}, top_level::{TopLevelSnapshot, TopLevelState, computer::MEMORY_SIZE, instruction::Instruction, }}, render::RenderMessage}, snapshot_manager::{self, Consumer, Producer}};
use nalgebra_glm as glm;

pub enum LogicMessage {
    PushRobot([Instruction; MEMORY_SIZE], glm::Vec2),
    StartRobotWatch(usize),
    StopRobotWatch(usize),
}

pub enum PhysicsMessage {
    PushRobot(RobotPhysics),
    ModifyChunk(Vec<(glm::IVec2, bool)>),
}


#[derive(Debug)]
pub struct JoinState {
    pub physics: Consumer<PhysicsSnapshot>,
    pub logic: Consumer<TopLevelSnapshot>,
    running: Arc<AtomicBool>,
    physics_tx: mpsc::Sender<PhysicsMessage>,
    logic_tx: mpsc::Sender<LogicMessage>,
}

trait SimulationState: Send + 'static {
    type Snapshot: Send + Sync + 'static;
    type Message: Send + 'static;
    fn update(&mut self, delta: f32, rx: &mpsc::Receiver<Self::Message>) -> Self::Snapshot;
}

impl JoinState {
    fn start_thread<T: SimulationState>(
        running: Arc<AtomicBool>,
        period: Duration,
        mut sim_state: T,
        mut producer: Producer<T::Snapshot>,
        rx: mpsc::Receiver<T::Message>,
    ) {
        thread::Builder::new()
            .spawn(move || {
                let delta = period.as_secs_f32();
                while running.load(Ordering::Relaxed) {
                    let start = Instant::now();
                    producer.publish(sim_state.update(delta, &rx));
                    thread::sleep(period.saturating_sub(start.elapsed()));
                }
            })
            .unwrap();
    }

    pub fn new(render_tx: mpsc::Sender<RenderMessage>) -> Self {
        let (logic_producer, logic_consumer) = snapshot_manager::channel(TopLevelSnapshot::new());
        let (physics_producer, physics_consumer) = snapshot_manager::channel(PhysicsSnapshot::new());
        let running = Arc::new(AtomicBool::new(true));

        let (physics_tx, physics_rx) = mpsc::channel();
        let (logic_tx, logic_rx) = mpsc::channel();

        let logic_state = TopLevelState::new(physics_consumer.clone(), render_tx);
        Self::start_thread(
            running.clone(),
            Duration::from_millis(100),
            logic_state, logic_producer, logic_rx,
        );

        let physics_state = PhysicsState::new(logic_consumer.clone());
        Self::start_thread(
            running.clone(),
            Duration::from_millis(20),
            physics_state, physics_producer, physics_rx,
        );

        Self {
            physics: physics_consumer,
            logic: logic_consumer,
            running,
            physics_tx,
            logic_tx,
        }
    }

    pub fn push_robot(&self, instructions: [Instruction; MEMORY_SIZE], position: glm::Vec2) {
        self.logic_tx.send(LogicMessage::PushRobot(instructions, position)).unwrap();
        self.physics_tx.send(PhysicsMessage::PushRobot(RobotPhysics { position, speed: glm::vec2(0.0, 0.0) })).unwrap();
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn get_physics_tx(&self) -> mpsc::Sender<PhysicsMessage> {
        self.physics_tx.clone()
    }

    pub fn get_logic_tx(&self) -> mpsc::Sender<LogicMessage> {
        self.logic_tx.clone()
    }
}