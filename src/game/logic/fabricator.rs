
use nalgebra_glm as glm;

use crate::{MainContext, game::logic::{computer::MEMORY_SIZE, instruction::Instruction, robot::Robot, world::{CHUNK_SIZE, MinableGrid}}, graphics::renderer::positioning::{PositionMode, RenderBox}, stable_index_vector::StableIndexVector};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputOutputDirection {
    BottomTop, LeftTop, RightTop,
    TopBottom, LeftBottom, RightBottom,
    TopLeft, BottomLeft, RightLeft,
    TopRight, BottomRight, LeftRight,
}

impl InputOutputDirection {
    pub fn separate(&self) -> (Direction, Direction) {
        match self {
            Self::BottomTop    => (Direction::Bottom, Direction::Top),
            Self::LeftTop      => (Direction::Left,   Direction::Top),
            Self::RightTop     => (Direction::Right,  Direction::Top),
            
            Self::TopBottom    => (Direction::Top,    Direction::Bottom),
            Self::LeftBottom   => (Direction::Left,   Direction::Bottom),
            Self::RightBottom  => (Direction::Right,  Direction::Bottom),
            
            Self::TopLeft      => (Direction::Top,    Direction::Left),
            Self::BottomLeft   => (Direction::Bottom, Direction::Left),
            Self::RightLeft    => (Direction::Right,  Direction::Left),
            
            Self::TopRight     => (Direction::Top,    Direction::Right),
            Self::BottomRight  => (Direction::Bottom, Direction::Right),
            Self::LeftRight    => (Direction::Left,   Direction::Right),
        }
    }
}


const PHASE_TIME: f32 = 5.0;
pub const PHASE_AMOUNT: u8 = 16;

#[derive(Debug)]
pub struct Fabricator {
    pub io_dirs: InputOutputDirection,
    pub phase: u8,
    time_next_phase: f32,
    pub position: glm::IVec2,
    pub start_code: [Instruction; MEMORY_SIZE],
}

impl Fabricator {
    pub fn new(
        position: glm::IVec2,
        io_dirs: InputOutputDirection,
    ) -> Self {
        Self {
            io_dirs,
            phase: 0,
            time_next_phase: PHASE_TIME,
            position,
            start_code: [Instruction::default(); MEMORY_SIZE],
        }
    }

    pub fn with_code(mut self, code: [Instruction; MEMORY_SIZE]) -> Self {
        self.start_code = code;
        self
    }

    pub fn update(&mut self, context: &mut MainContext, grid: &mut MinableGrid, robots: &mut StableIndexVector<Robot>) {
        let delta = context.ui.get_delta();
        
        self.time_next_phase -= delta;
        if self.time_next_phase <= 0.0 {
            let (input, output) = self.io_dirs.separate();
            let input_pos = self.position + input.get_offset();
            let output_pos = self.position + output.get_offset();
            let output_bbox = Robot::get_render_box(output_pos.map(|x| x as f32));

            if self.phase != 0 || robots.iter_unordered()
                .fold(true, |prev, robot| {
                    let bbox = Robot::get_render_box(robot.movement_module.get_position());
                    prev && !bbox.intersects(&output_bbox)
                })
            {
                let input_chunk_pos = input_pos.map(|x| x.rem_euclid(CHUNK_SIZE as i32) as u8);
                if grid.try_mine(input_chunk_pos) {
                    self.phase += 1;
                    self.time_next_phase += PHASE_TIME;                
                }

                if self.phase == PHASE_AMOUNT {
                    self.phase = 0;
                    
                    robots.push(Robot::new(output_pos.map(|x| x as f32), self.start_code));
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

    pub fn get_code(&self) -> &[Instruction; MEMORY_SIZE] {
        &self.start_code
    }

    pub fn get_possible_ghost_position(&self) -> glm::Vec2 {
        let (_, output) = self.io_dirs.separate();
        (self.position + output.get_offset()).map(|x| x as f32)
    }

    pub fn get_ghost_position(&self) -> Option<glm::Vec2> {
        (self.phase != 0).then_some(self.get_possible_ghost_position())
    }
}
