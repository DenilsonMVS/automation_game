
use crate::game::{instruction::Syscall, world::World};
use nalgebra_glm as glm;
pub const MINING_TIME: f32 = 1.0;

#[derive(Debug, Clone, Copy)]
pub struct MiningComputer {
    syscall: Syscall,
    stored: u8,
    max_storage: u8,
    remaining_time: f32,
}

impl MiningComputer {
    pub fn new(max_storage: u8) -> Self {
        Self { stored: 0, max_storage, syscall: Syscall::Nop, remaining_time: 0.0, }
    }

    pub fn receive_syscall(&mut self, syscall: Syscall) {
        self.syscall = syscall;
        match self.syscall {
            Syscall::Mine => self.remaining_time = MINING_TIME,
            _ => {},
        }
    }

    pub fn update(&mut self, position: glm::Vec2, world: &mut World, delta: f32, is_instruction_tick: bool) -> Option<u8> {
        match self.syscall {
            Syscall::Mine => {
                if delta >= self.remaining_time {
                    self.remaining_time = 0.0;
                    match self.syscall {
                        Syscall::Mine => {
                            if world.try_mine(position.map(|x| x.round() as i32)) {
                                self.stored += 1;
                                Some(1)
                            } else {
                                Some(0)
                            }
                        },
                        _ => Some(0),
                    }
                } else {
                    self.remaining_time -= delta;
                    None
                }
            },
            Syscall::MaxStorage => is_instruction_tick.then_some(self.max_storage),
            Syscall::Stored => is_instruction_tick.then_some(self.stored),
            _ => None,
        }
    }
}