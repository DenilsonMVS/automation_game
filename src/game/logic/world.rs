
use std::{array, marker::PhantomData, ops::{Add, Deref, Sub}};
use nalgebra_glm::{self as glm, Scalar};

use crate::{MainContext, game::logic::{fabricator::Fabricator, robot::Robot}, open_simplex::{FractalSample, OpenSimplex}, stable_index_vector::StableIndexVector};

pub const CHUNK_MAG: usize = 8;
pub const CHUNK_SIZE: usize = 1 << CHUNK_MAG;


pub trait SpaceIdentifier {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GenericPosition<T: Clone + Copy + Scalar, Label: SpaceIdentifier> {
    pub position: glm::TVec2<T>,
    phantom: PhantomData<Label>,
}

impl<T: Clone + Copy + Scalar, Label: SpaceIdentifier> Deref for GenericPosition<T, Label> {
    type Target = glm::TVec2<T>;
    fn deref(&self) -> &Self::Target {
        &self.position
    }
}

impl<T: Clone + Copy + Scalar, Label: SpaceIdentifier> GenericPosition<T, Label> {
    pub fn new(x: T, y: T) -> Self {
        Self {
            position: glm::vec2(x, y),
            phantom: PhantomData,
        }
    }

    pub fn map<O, F>(&self, f: F) -> GenericPosition<O, Label>
    where
        O: Clone + Copy + Scalar,
        F: FnMut(T) -> O,
    {
        GenericPosition {
            position: self.position.map(f),
            phantom: PhantomData,
        }
    }
}

macro_rules! impl_op {
    ($trait:ident, $func:ident) => {
        impl<T, Label: SpaceIdentifier> $trait<GenericPosition<T, Label>> for GenericPosition<T, Label>
        where
            T: Scalar + Copy,
            glm::TVec2<T>: $trait<Output = glm::TVec2<T>>,
        {
            type Output = Self;

            fn $func(self, rhs: Self) -> Self::Output {
                GenericPosition {
                    position: $trait::$func(self.position, rhs.position),
                    phantom: PhantomData,
                }
            }
        }
    };
}

impl_op!(Add, add);
impl_op!(Sub, sub);

pub enum Spaces {
    Chunk,
    Global,
}

pub mod spaces {
    use crate::game::logic::world::SpaceIdentifier;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Chunk;
    impl SpaceIdentifier for Chunk {}

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ChunkId;
    impl SpaceIdentifier for ChunkId {}

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Global;
    impl SpaceIdentifier for Global {}
}

pub type ChunkPosition<T> = GenericPosition<T, spaces::Chunk>;
pub type ChunkId = GenericPosition<i32, spaces::Chunk>;
pub type GlobalPosition<T> = GenericPosition<T, spaces::Global>;

impl ChunkId {
    fn to_global(&self) -> GlobalPosition<i32> {
        GlobalPosition {
            position: self.position * CHUNK_SIZE as i32,
            phantom: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridPosition {
    None,
    Resource(u8),
    Obstacle,
}

impl Default for GridPosition {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug)]
pub struct Chunk {
    grid: [[GridPosition; CHUNK_SIZE]; CHUNK_SIZE],
    pending_updates: Vec<ChunkPosition<u8>>,
    pub fabricators: Vec<Fabricator>,
    pub robots: StableIndexVector<Robot>,
}

pub struct MinableGrid<'a> {
    grid: &'a mut [[GridPosition; CHUNK_SIZE]; CHUNK_SIZE],
    pending_updates: &'a mut Vec<ChunkPosition<u8>>,
}

impl<'a> MinableGrid<'a> {
    pub fn try_mine(&mut self, position: ChunkPosition<u8>) -> bool {
        let el = &mut self.grid[position.y as usize][position.x as usize];
        return match el {
            GridPosition::Resource(quantity) => {
                if *quantity == 1 {
                    *el = GridPosition::None;
                } else {
                    *quantity -= 1;
                }
                self.pending_updates.push(position);
                return true;
            },
            _ => false,
        };
    }

    pub fn try_release(&mut self, position: ChunkPosition<u8>) -> bool {
        let el = &mut self.grid[position.y as usize][position.x as usize];
        match el {
            GridPosition::Resource(quantity) => {
                if *quantity == u8::MAX {
                    false
                } else {
                    *quantity += 1;
                    self.pending_updates.push(position);
                    true
                }
            },
            GridPosition::None => {
                *el = GridPosition::Resource(1);
                self.pending_updates.push(position);
                true
            }
            _ => false,
        }
    }

    pub fn get(&self, position: ChunkPosition<u8>) -> GridPosition {
        self.grid[position.y as usize][position.x as usize]
    }
}

impl Chunk {
    pub fn new(fabricators: Vec<Fabricator>, robots: StableIndexVector<Robot>) -> Self {
        Self {
            grid: [[GridPosition::None; _]; _],
            pending_updates: vec![],
            fabricators, robots,
        }
    }

    pub fn new_proc_generated(chunk_id: ChunkId, noise: &OpenSimplex) -> Self {
        let start_position = chunk_id.to_global().map(|x| x as f32);
        let grid = array::from_fn(|line| array::from_fn(|column| {
            let current_position = start_position + GlobalPosition::new(column as f32, line as f32);
            let value = noise.fractal_sample(current_position.position, &FractalSample::new_harmonic::<6>(255.0, 0.05, 0.5, 2.0));
            if value <= 0.0 {
                GridPosition::None
            } else {
                GridPosition::Resource((value as u8).max(1))
            }
        }));

        return Self {
            grid,
            pending_updates: vec![],
            fabricators: vec![],
            robots: StableIndexVector::new(),
        };
    }

    pub fn get_pending_updates(&self) -> &[ChunkPosition<u8>] {
        &self.pending_updates
    }

    pub fn get_local_position(&self, position: ChunkPosition<u8>) -> GridPosition {
        return self.grid[position.y as usize][position.x as usize];
    }

    pub fn get_grid(&self) -> &[[GridPosition; CHUNK_SIZE]; CHUNK_SIZE] {
        &self.grid
    }

    pub fn get_grid_mut(&mut self) -> &mut [[GridPosition; CHUNK_SIZE]; CHUNK_SIZE] {
        &mut self.grid
    }

    pub fn update(&mut self, context: &mut MainContext) {
        self.pending_updates.clear();

        let mut minable_grid = MinableGrid {
            grid: &mut self.grid,
            pending_updates: &mut self.pending_updates,
        };

        for robot in self.robots.iter_unordered_mut() {
            robot.update(&mut minable_grid, context);
        }

        for fabricator in self.fabricators.iter_mut() {
            fabricator.update(context, &mut minable_grid, &mut self.robots);
        }

        self.handle_collisions();
    }
}
