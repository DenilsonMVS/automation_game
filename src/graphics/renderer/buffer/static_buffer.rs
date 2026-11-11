use std::{ffi::c_void, marker::PhantomData};
use crate::graphics::renderer::{buffer::Buffer, vertex_array_object::{StaticVertexLayout, VertexLayout}};


pub(crate) struct StaticBuffer<T> {
    id: u32,
    _marker: PhantomData<T>,
}

impl<T> StaticBuffer<T> {
    const CHECK_NON_ZERO_SIZE: () = assert!(
        std::mem::size_of::<T>() > 0,
        "T cannot be a zero-sized type (ZST)",
    );

    pub(crate) fn new(data: &[T]) -> Self {
        let _ = Self::CHECK_NON_ZERO_SIZE;

        let mut id = 0u32;
        unsafe {
            gl::CreateBuffers(1, &mut id);
            gl::NamedBufferStorage(
                id,
                (std::mem::size_of::<T>() * data.len()) as isize,
                data.as_ptr() as *const c_void,
                0
            );
        }

        return Self {
            id,
            _marker: PhantomData
        };
    }
}

impl<T> Buffer for StaticBuffer<T> {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn bind_as_storage(&self, binding_point: u32) {
        unsafe {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, binding_point, self.id);
        }
    }
}

impl<T> Drop for StaticBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

impl<T: StaticVertexLayout> VertexLayout for StaticBuffer<T> {
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
