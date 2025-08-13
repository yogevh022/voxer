use super::{RendererBuilder, resources};

impl RendererBuilder<'_> {
    pub fn make_vertex_buffer(&self, size: u64) -> wgpu::Buffer {
        resources::vertex::create_buffer(self.device.as_ref().unwrap(), size)
    }

    pub fn make_index_buffer(&self, size: u64) -> wgpu::Buffer {
        resources::index::create_buffer(self.device.as_ref().unwrap(), size)
    }

    pub fn make_indirect_buffer(&self, size: u64) -> wgpu::Buffer {
        todo!()
    }
}
