use std::num::NonZeroU32;

use crate::graphics::renderer::{buffer::{Buffer, dynamic_buffer::DynamicBuffer}, vertex_array_object::{StaticVertexLayout, VertexLayout}};

pub(crate) struct StreamBuffer<T> {
    dynamic_buffer: DynamicBuffer<T>,
    cursor_pos: u32,
    needs_rebind: bool,
}

impl<T> StreamBuffer<T> {
    pub(crate) fn new(initial_capacity: NonZeroU32) -> Self {
        Self {
            dynamic_buffer: DynamicBuffer::new(initial_capacity),
            cursor_pos: 0,
            needs_rebind: false,
        }
    }

    pub(crate) fn push(&mut self, value: &T) {
        if self.cursor_pos >= self.dynamic_buffer.get_count() {
            self.dynamic_buffer = self.dynamic_buffer.clone_with_capacity(NonZeroU32::new(self.cursor_pos * 2).unwrap());
            self.needs_rebind = true;
        }

        self.dynamic_buffer.write(value, self.cursor_pos);
        self.cursor_pos += 1;
    }

    pub(crate) fn take_rebind(&mut self) -> bool {
        let previous_value = self.needs_rebind;
        self.needs_rebind = false;
        previous_value
    }

    pub(crate) fn reset(&mut self) {
        self.cursor_pos = 0;
    }

    pub(crate) fn get_cursor_pos(&self) -> u32 {
        self.cursor_pos
    }
}

impl<T> Buffer for StreamBuffer<T> {
    fn bind_as_storage(&self, binding_point: u32) {
        self.dynamic_buffer.bind_as_storage(binding_point);
    }

    fn get_id(&self) -> u32 {
        self.dynamic_buffer.get_id()
    }
}

impl<T: StaticVertexLayout> VertexLayout for StreamBuffer<T> {
    fn get_divisor(&self) -> u32 {
        T::get_divisor()
    }

    fn get_fields(&self) -> &'static [crate::graphics::renderer::vertex_array_object::FieldType] {
        T::get_fields()
    }

    fn get_stride(&self) -> i32 {
        T::get_stride()
    }
}
