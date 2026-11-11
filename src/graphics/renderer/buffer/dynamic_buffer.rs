use std::{num::NonZeroU32, os::raw::c_void, ptr::NonNull};

use crate::graphics::renderer::{buffer::Buffer, vertex_array_object::{StaticVertexLayout, VertexLayout}};


#[derive(Debug)]
pub(crate) struct DynamicBuffer<T> {
    id: NonZeroU32,
    ptr: NonNull<T>,
    count: NonZeroU32,
}

impl<T> Buffer for DynamicBuffer<T> {
    fn get_id(&self) -> u32 {
        self.id.get()
    }

    fn bind_as_storage(&self, binding_point: u32) {
        unsafe {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, binding_point, self.id.get());
        }
    }
}

impl<T> Drop for DynamicBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::UnmapNamedBuffer(self.id.get());
            gl::DeleteBuffers(1, &self.id.get());
        }
    }
}

impl<T> DynamicBuffer<T> {
    const CHECK_NON_ZERO_SIZE: () = assert!(
        std::mem::size_of::<T>() > 0,
        "T cannot be a zero-sized type (ZST)",
    );

    pub(crate) fn new(count: NonZeroU32) -> Self {
        let _ = Self::CHECK_NON_ZERO_SIZE;

        let id ;
        let mut raw_id = 0u32;
        let byte_size = (std::mem::size_of::<T>() * count.get() as usize) as isize;

        let storage_flags = gl::MAP_WRITE_BIT
            | gl::MAP_PERSISTENT_BIT
            | gl::MAP_COHERENT_BIT
            | gl::DYNAMIC_STORAGE_BIT;
            
        let map_flags = gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT;

        let raw_ptr;
        unsafe {
            gl::CreateBuffers(1, &mut raw_id);
            id = NonZeroU32::new(raw_id).unwrap();
            gl::NamedBufferStorage(id.get(), byte_size, std::ptr::null(), storage_flags);
            raw_ptr = gl::MapNamedBufferRange(id.get(), 0, byte_size, map_flags) as *mut T;
        }

        let ptr = NonNull::new(raw_ptr).unwrap();

        Self {
            id,
            ptr,
            count,
        }
    }

    pub(crate) fn new_initialized(data: &[T]) -> Self {
        let _ = Self::CHECK_NON_ZERO_SIZE;
        let count = NonZeroU32::new(data.len() as u32).unwrap();

        let id ;
        let mut raw_id = 0u32;
        let byte_size = (std::mem::size_of::<T>() * data.len()) as isize;

        let storage_flags = gl::MAP_WRITE_BIT
            | gl::MAP_PERSISTENT_BIT
            | gl::MAP_COHERENT_BIT
            | gl::DYNAMIC_STORAGE_BIT;
            
        let map_flags = gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT;

        let raw_ptr;
        unsafe {
            gl::CreateBuffers(1, &mut raw_id);
            id = NonZeroU32::new(raw_id).unwrap();
            gl::NamedBufferStorage(id.get(), byte_size, data.as_ptr() as * const c_void, storage_flags);
            raw_ptr = gl::MapNamedBufferRange(id.get(), 0, byte_size, map_flags) as *mut T;
        }

        let ptr = NonNull::new(raw_ptr).unwrap();

        Self {
            id,
            ptr,
            count,
        }
    }

    pub(crate) fn write(&mut self, value: &T, position: u32) {
        unsafe {
            debug_assert!(position < self.count.get());
            let dst = self.ptr.add(position as usize);
            std::ptr::copy_nonoverlapping(value as *const T, dst.as_ptr(), 1);
        }
    }

    pub(crate) fn get_count(&self) -> u32 {
        self.count.get()
    }

    pub(crate) fn clone_with_capacity(&self, new_count: NonZeroU32) -> Self {
        let new_buffer = Self::new(new_count);
        let amount_to_copy = self.count.min(new_count);
        let bytes_to_copy = (std::mem::size_of::<T>() * amount_to_copy.get() as usize) as isize;
        unsafe {
            gl::CopyNamedBufferSubData(self.id.get(), new_buffer.id.get(), 0, 0, bytes_to_copy);
        }
        new_buffer
    }
}

impl<T: StaticVertexLayout> VertexLayout for DynamicBuffer<T> {
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
