
use std::collections::HashSet;
use nalgebra_glm as glm;
use crate::game::logic::robot::Robot;

#[derive(Debug)]
pub struct Targets {
    targets: HashSet<glm::U8Vec2>,
}

impl Targets {
    pub fn new(targets: HashSet<glm::U8Vec2>) -> Self {
        Self { targets }
    }

    pub fn count<'a>(&self, robots: impl Iterator<Item = &'a Robot>) -> (usize, usize) {
        (
            robots.fold(0, |prev, robot| {
                let ipos = robot.movement_module.get_position().map(|x| x.round() as u8);
                if self.targets.contains(&ipos) {
                    prev + 1
                } else {
                    prev
                }
            }),
            self.targets.len(),
        )
    }

    pub fn get_targets(&self) -> &HashSet<glm::U8Vec2> {
        &self.targets
    }
}
