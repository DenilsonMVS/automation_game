use std::num::NonZeroU32;

use crate::graphics::renderer::{buffer::{Buffer, dynamic_buffer::DynamicBuffer}, vertex_array_object::{FieldType, StaticVertexLayout, VertexLayout}};

const BLOCK_SIZE: u32 = 0x100;
const INITIAL_BLOCK_AMOUNT: u32 = 0x100;

#[derive(Debug)]
struct Cursor {
    blocks: Vec<u32>,
    cursor_position_in_current_block: u32,
}

impl Cursor {
    fn get_global_cursor_position(&self) -> u32 {
        self.blocks.last().unwrap() * BLOCK_SIZE + self.cursor_position_in_current_block
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CursorBlock {
    pub(crate) block_start: u32,
    pub(crate) used_size: u32,
}

pub(crate) struct BlockBuffer<T> {
    dynamic_buffer: DynamicBuffer<T>,
    cursors: Vec<Cursor>,
    needs_rebind: bool,
    total_assigned_blocks: u32,
}

impl<T> BlockBuffer<T> {
    pub(crate) fn new() -> Self {
        Self::with_capacity(NonZeroU32::new(INITIAL_BLOCK_AMOUNT).unwrap())
    }

    pub(crate) fn with_capacity(initial_block_amount: NonZeroU32) -> Self {
        let dynamic_buffer = DynamicBuffer::new(NonZeroU32::new(BLOCK_SIZE * initial_block_amount.get()).unwrap());
        Self {
            dynamic_buffer,
            cursors: vec![],
            needs_rebind: false,
            total_assigned_blocks: 0
        }
    }

    pub(crate) fn push_cursor(&mut self) -> u32 {
        let cursor_id = self.cursors.len() as u32;
        self.cursors.push(Cursor { blocks: vec![], cursor_position_in_current_block: BLOCK_SIZE });
        cursor_id
    }

    fn push_block(&mut self, cursor_id: u32) {
        let current_total_usage = self.total_assigned_blocks * BLOCK_SIZE;
        if current_total_usage == self.dynamic_buffer.get_count() {
            self.dynamic_buffer = self.dynamic_buffer.clone_with_capacity(NonZeroU32::new(current_total_usage * 2).unwrap());
            self.needs_rebind = true;
        }

        let cursor = &mut self.cursors[cursor_id as usize];
        cursor.blocks.push(self.total_assigned_blocks);
        cursor.cursor_position_in_current_block = 0;

        self.total_assigned_blocks += 1;
    }

    pub(crate) fn take_rebind(&mut self) -> bool {
        let previous_value = self.needs_rebind;
        self.needs_rebind = false;
        previous_value
    }

    pub(crate) fn push(&mut self, cursor_id: u32, value: &T) {
        if self.cursors[cursor_id as usize].cursor_position_in_current_block == BLOCK_SIZE {
            self.push_block(cursor_id)
        }

        let cursor = &mut self.cursors[cursor_id as usize];
        self.dynamic_buffer.write(value, cursor.get_global_cursor_position());
        cursor.cursor_position_in_current_block += 1;
    }

    pub(crate) fn get_cursor_blocks(&self, cursor_id: u32) -> impl Iterator<Item = CursorBlock> {
        let cursor = &self.cursors[cursor_id as usize];
        let block_amount = cursor.blocks.len();
        let last_block_size = cursor.cursor_position_in_current_block as u32;
        cursor.blocks
            .iter()
            .enumerate()
            .map(move |(i, block_id)| CursorBlock {
                block_start: (*block_id * BLOCK_SIZE) as u32,
                used_size: if i + 1 == block_amount {
                    last_block_size
                } else {
                    BLOCK_SIZE as u32
                },
            })
    }

    pub(crate) fn reset(&mut self) {
        self.total_assigned_blocks = 0;
        self.cursors.clear();
    }

    pub(crate) fn get_cursor_count(&self) -> usize {
        self.cursors.len()
    }
} 

impl<T> Buffer for BlockBuffer<T> {
    fn bind_as_storage(&self, binding_point: u32) {
        self.dynamic_buffer.bind_as_storage(binding_point);
    }

    fn get_id(&self) -> u32 {
        self.dynamic_buffer.get_id()
    }
}

impl<T: StaticVertexLayout> VertexLayout for BlockBuffer<T> {
    fn get_divisor(&self) -> u32 {
        T::get_divisor()
    }

    fn get_fields(&self) -> &'static [FieldType] {
        T::get_fields()
    }

    fn get_stride(&self) -> i32 {
        T::get_stride()
    }
}
