
use crate::graphics::renderer::{GlEnum, buffer::Buffer, draw_mode::DrawMode, types::{GlType, GlTypeEnum}};


#[repr(C)]
pub(crate) struct DrawArraysIndirectCommand {
    pub(crate) count: u32,
    pub(crate) instance_count: u32,
    pub(crate) first: u32,
    pub(crate) base_instance: u32,
}

pub(crate) struct FieldType {
    pub(crate) gl_type: GlTypeEnum,
    pub(crate) size: i32,
    pub(crate) normalized: bool,
    pub(crate) offset: u32,
}

impl FieldType {
    pub(crate) const fn new<T: GlType>(normalized: bool, offset: u32) -> Self {
        Self {
            normalized,
            size: T::FIELD_TYPE_SIZE,
            gl_type: T::ENUM,
            offset,
        }
    }
}

pub(crate) trait StaticVertexLayout: Sized {
    fn get_fields() -> &'static [FieldType];
    fn get_stride() -> i32 {
        std::mem::size_of::<Self>() as i32
    }
    fn get_divisor() -> u32;
}

pub(crate) trait VertexLayout: Buffer {
    fn get_fields(&self) -> &'static [FieldType];
    fn get_stride(&self) -> i32;
    fn get_divisor(&self) -> u32 { 0 }
}

#[derive(Debug)]
pub(crate) struct VertexArrayObject {
    id: u32,
}

impl VertexArrayObject {
    pub(crate) fn new(buffers: &[& dyn VertexLayout]) -> Self {
        let mut id: u32 = 0u32;
        unsafe {
            gl::CreateVertexArrays(1, &mut id);
        }

        let mut attribute_index = 0;
        for (binding_index, buffer) in buffers.iter().enumerate() {
            let binding_index = binding_index as u32;
            let fields = buffer.get_fields();
            let stride = buffer.get_stride();
            let divisor = buffer.get_divisor();

            unsafe {
                gl::VertexArrayVertexBuffer(
                    id,
                    binding_index,
                    buffer.get_id(),
                    0,
                    stride as i32
                );

                gl::VertexArrayBindingDivisor(id, binding_index, divisor);
            }

            for field in fields {
                unsafe {
                    gl::EnableVertexArrayAttrib(id, attribute_index);

                    let use_iformat = !field.normalized && field.gl_type.is_integer();
                    if use_iformat {
                        gl::VertexArrayAttribIFormat(
                            id,
                            attribute_index,
                            field.size,
                            field.gl_type.to_gl_enum(),
                            field.offset
                        );
                    } else {
                        gl::VertexArrayAttribFormat(
                            id,
                            attribute_index,
                            field.size,
                            field.gl_type.to_gl_enum(),
                            field.normalized as u8,
                            field.offset
                        );
                    }

                    gl::VertexArrayAttribBinding(id, attribute_index, binding_index);
                }

                attribute_index += 1;
            }
        }

        return Self { id };
    }

    pub(crate) fn rebind(&self, binding_index: u32, buffer: &dyn VertexLayout) {
        unsafe {
            gl::VertexArrayVertexBuffer(
                self.id,
                binding_index,
                buffer.get_id(),
                0,
                buffer.get_stride(),
            );
        }
    }

    pub(crate) fn draw_instanced(&self, vertex_count: i32, instance_count: i32, draw_mode: DrawMode) {
		unsafe {
            gl::BindVertexArray(self.id);
			gl::DrawArraysInstanced(
				draw_mode.to_gl_enum(),
				0,
				vertex_count,
				instance_count
			);
		}
	}

    pub(crate) fn draw(&self, vertex_count: i32, draw_mode: DrawMode) {
        unsafe {
            gl::BindVertexArray(self.id);            
            gl::DrawArrays(
                draw_mode.to_gl_enum(),
                0,
                vertex_count
            );
        }
    }

    pub(crate) fn draw_multi_instanced(&self,
        command_buffer: &dyn Buffer,
        command_start: u32,
        amount_commands: u32,
        draw_mode: DrawMode,
    ) {
        unsafe {
            gl::BindVertexArray(self.id);
            gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, command_buffer.get_id());
            let offset_bytes = command_start as usize * std::mem::size_of::<DrawArraysIndirectCommand>();
            let offset_ptr = offset_bytes as *const std::ffi::c_void;
            gl::MultiDrawArraysIndirect(
                draw_mode.to_gl_enum(),
                offset_ptr,
                amount_commands as i32,
                0,
            );
        }
    }
}

impl Drop for VertexArrayObject {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &mut self.id);
        }
    }
}
