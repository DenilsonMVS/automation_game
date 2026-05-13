
use nalgebra_glm as glm;

use crate::game::logic::instruction::Syscall;

const POSITION_TOLERANCE: f32 = 0.02;
const SPEED_TOLERANCE: f32 = 0.01;

#[derive(Debug, Clone, Copy)]
pub struct MovementModule {
    prev_position: glm::Vec2,
    position: glm::Vec2,
    speed: glm::Vec2,
    target: glm::Vec2,
}

impl MovementModule {
    pub fn new(position: glm::Vec2) -> Self {
        Self {
            prev_position: position,
            position,
            speed: glm::vec2(0.0, 0.0),
            target: position,
        }
    }

    pub fn set_position(&mut self, position: glm::Vec2) {
        self.position = position;
    }

    pub fn update_sensor(&mut self, position: glm::Vec2, speed: glm::Vec2) {
        self.prev_position = self.position;
        self.position = position;
        self.speed = speed;
    }

    pub fn update(&mut self) -> Option<u8> {
        self.target_achieved().then_some(1)
    }

    fn target_achieved(&self) -> bool {
        let sq = |x: f32| x * x;

        let diff = self.target - self.position;
        let distance2 = glm::length2(&diff);
        let speed_val2 = glm::length2(&self.speed);

        return distance2 <= sq(POSITION_TOLERANCE) && speed_val2 <= sq(SPEED_TOLERANCE);
    }

    pub fn calculate_accelleration(&self) -> glm::Vec2 {
        const PROPORTIONAL: f32 = 10.0;
        const DERIVATIVE: f32 = -6.0;

        let distance = self.target - self.position;
        let d = self.speed * DERIVATIVE;
        let p = distance * PROPORTIONAL;
        return p + d;
    }

    pub fn get_position(&self) -> glm::Vec2 {
        self.position
    }

    pub fn get_prev_position(&self) -> glm::Vec2 {
        self.prev_position
    }

    pub fn get_speed(&self) -> glm::Vec2 {
        self.speed
    }

    pub fn receive_syscall(&mut self, sys: Syscall) {
        self.target = self.position.map(|x| x.round()) + match sys {
            Syscall::MoveBottom => glm::vec2(0.0, -1.0),
            Syscall::MoveLeft => glm::vec2(-1.0, 0.0),
            Syscall::MoveRight => glm::vec2(1.0, 0.0),
            Syscall::MoveTop => glm::vec2(0.0, 1.0),
            _ => glm::vec2(0.0, 0.0),
        };
    }
}
