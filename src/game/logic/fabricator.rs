
use nalgebra_glm as glm;

use crate::{MainContext, game::logic::{robot::Robot, world::World}, graphics::renderer::positioning::{PositionMode, RenderBox}, stable_index_vector::StableIndexVector};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Top, Bottom, Left, Right
}

impl Direction {
    pub fn get_offset(&self) -> glm::IVec2 {
        match self {
            Self::Bottom => glm::vec2( 0, -1),
            Self::Top    => glm::vec2( 0,  1),
            Self::Left   => glm::vec2(-1,  0),
            Self::Right  => glm::vec2( 1,  0),
        }
    }
}

const PHASE_TIME: f32 = 5.0;
pub const PHASE_AMOUNT: u16 = 16;

#[derive(Debug)]
pub struct Fabricator {
    pub input: Direction,
    pub output: Direction,
    pub phase: u16,
    time_next_phase: f32,
    pub position: glm::IVec2,
}

impl Fabricator {
    pub fn new(
        position: glm::IVec2,
        input: Direction,
        output: Direction,
    ) -> Self {
        Self { input, output, phase: 0, time_next_phase: PHASE_TIME, position }
    }

    pub fn update(&mut self, context: &mut MainContext, world: &mut World, robots: &mut StableIndexVector<Robot>) {
        let delta = context.ui.get_delta();
        
        self.time_next_phase -= delta;
        if self.time_next_phase <= 0.0 {
            let input_pos = self.position + self.input.get_offset();
            let output_pos = self.position + self.output.get_offset();
            let output_bbox = Robot::get_render_box(output_pos.map(|x| x as f32));

            if self.phase != 0 || robots.iter_unordered()
                .fold(true, |prev, robot| {
                    let bbox = Robot::get_render_box(robot.movement_module.get_position());
                    prev && !bbox.intersects(&output_bbox)
                })
            {
                if world.try_mine(input_pos) {
                    self.phase += 1;
                    self.time_next_phase += PHASE_TIME;                
                }

                if self.phase == PHASE_AMOUNT {
                    self.phase = 0;
                    
                    robots.push(Robot::new(output_pos.map(|x| x as f32), [Default::default(); _]));
                }
            }
        }

        self.time_next_phase = self.time_next_phase.max(0.0);
    }

    pub fn get_bbox(position: glm::Vec2) -> RenderBox {
        RenderBox::new_simple(
            position.map(|x| x as f32),
            PositionMode::Center,
            glm::vec2(1.0, 1.0)
        )
    }

    pub fn get_possible_ghost_position(&self) -> glm::Vec2 {
        (self.position + self.output.get_offset()).map(|x| x as f32)
    }

    pub fn get_ghost_position(&self) -> Option<glm::Vec2> {
        (self.phase != 0).then_some(self.get_possible_ghost_position())
    }
}
