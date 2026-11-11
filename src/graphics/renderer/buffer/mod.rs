
pub mod static_buffer;
pub mod dynamic_buffer;
pub mod stream_buffer;
pub mod block_buffer;

pub(crate) trait Buffer {
    fn bind_as_storage(&self, binding_point: u32);
    fn get_id(&self) -> u32;
}