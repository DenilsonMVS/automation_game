
use nalgebra_glm as glm;

use crate::game::logic::{instruction::Syscall};

const MINING_TIME: f32 = 10.0;
const SCAN_TIME: f32 = 1.0;
const RELEASE_TIME: f32 = 2.0;

#[derive(Debug, Clone, Copy)]
pub struct MiningModule {
    syscall: Syscall,
    stored: u8,
    max_storage: u8,
    remaining_time: f32,
}

impl MiningModule {
    pub fn new(max_storage: u8) -> Self {
        Self { stored: 0, max_storage, syscall: Syscall::Nop, remaining_time: 0.0, }
    }

    pub fn get_stored(&self) -> u8 {
        self.stored
    }

    pub fn receive_syscall(&mut self, syscall: Syscall) {
        self.syscall = syscall;
        match self.syscall {
            Syscall::Mine => self.remaining_time = MINING_TIME,
            Syscall::Scan => self.remaining_time = SCAN_TIME,
            Syscall::Release => self.remaining_time = RELEASE_TIME,
            _ => {},
        }
    }

    // pub fn update(&mut self, position: glm::Vec2, grid: &mut MinableGrid, delta: f32, is_instruction_tick: bool) -> Option<u8> {
    //     let chunk_pos = position.map(|x| (x.round() as i32).rem_euclid(CHUNK_SIZE as i32) as u8);
    //     match self.syscall {
    //         Syscall::Mine => {
    //             if self.stored == self.max_storage {
    //                 is_instruction_tick.then_some(0)
    //             } else if delta >= self.remaining_time {
    //                 self.remaining_time = 0.0;
    //                 if grid.try_mine(chunk_pos) {
    //                     self.stored += 1;
    //                     Some(1)
    //                 } else {
    //                     Some(0)
    //                 }
    //             } else {
    //                 self.remaining_time -= delta;
    //                 None
    //             }
    //         },
    //         Syscall::MaxStorage => is_instruction_tick.then_some(self.max_storage),
    //         Syscall::Stored => is_instruction_tick.then_some(self.stored),
    //         Syscall::Scan => {
    //             if delta >= self.remaining_time {
    //                 self.remaining_time = 0.0;
    //                 let position = grid.get(chunk_pos);
    //                 match position {
    //                     GridPosition::Resource(amount) => Some(amount as u8),
    //                     _ => Some(0),
    //                 }
    //             } else {
    //                 self.remaining_time -= delta;
    //                 None
    //             }
    //         },
    //         Syscall::Release => {
    //             if delta >= self.remaining_time {
    //                 self.remaining_time = 0.0;
    //                 if self.stored == 0 {
    //                     return Some(0);
    //                 }

    //                 if grid.try_release(chunk_pos) {
    //                     self.stored -= 1;
    //                     Some(1)
    //                 } else {
    //                     Some(0)
    //                 }
    //             } else {
    //                 self.remaining_time -= delta;
    //                 None
    //             }
    //         },
    //         _ => None,
    //     }
    // }
}
