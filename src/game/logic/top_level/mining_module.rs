use crate::game::logic::top_level::{instruction::Syscall, world::World};
use nalgebra_glm as glm;

const MINING_TICKS: u8 = 40;
const SCAN_TICKS: u8 = 4;
const RELEASE_TICKS: u8 = 10;

// const MINING_TICKS: u8 = 1;
// const SCAN_TICKS: u8 = 1;
// const RELEASE_TICKS: u8 = 1;

#[derive(Debug, Clone, Copy)]
pub struct MiningModule {
    stored: u8,
    max_storage: u8,
    ticks_remaing: u8,
    syscall: Option<Syscall>,
}

impl MiningModule {
    pub fn new(max_storage: u8) -> Self {
        MiningModule {
            ticks_remaing: 0,
            syscall: None,
            stored: 0,
            max_storage,
        }
    }

    pub fn update(&mut self, world: &mut World, position: glm::Vec2) -> Option<u8> {
        let ipos = position.map(|x| x.round() as i32);
        self.ticks_remaing = self.ticks_remaing.saturating_sub(1);

        let Some(syscall) = self.syscall else { return None };
        match (syscall, self.ticks_remaing) {
            (Syscall::Mine, 0) => {
                if self.stored < self.max_storage && world.try_mine(ipos) {
                    self.stored += 1;
                    Some(1)
                } else {
                    Some(0)
                }
            },
            (Syscall::Stored, _) => Some(self.stored),
            (Syscall::MaxStorage, _) => Some(self.max_storage),
            (Syscall::Scan, 0) => Some(world.get_amount(ipos)),
            (Syscall::Release, 0) => todo!(),
            _ => None,
        }
    }

    pub fn receive_syscall(&mut self, syscall: Syscall) {
        // Syscall::Mine | Syscall::Stored | Syscall::MaxStorage | Syscall::Scan | Syscall::Release
        self.syscall = Some(syscall);
        match syscall {
            Syscall::Mine => self.ticks_remaing = MINING_TICKS,
            Syscall::Scan => self.ticks_remaing = SCAN_TICKS,
            Syscall::Release => self.ticks_remaing = RELEASE_TICKS,
            _ => {},
        }
    }
}
