use super::{RendererBuilder, resources};
use wgpu::BufferUsages;

impl RendererBuilder<'_> {
    pub fn make_vertex_buffer(&self, size: u64) -> wgpu::Buffer {
        resources::vertex::create_buffer(self.device.as_ref().unwrap(), size)
    }

    pub fn make_index_buffer(&self, size: u64) -> wgpu::Buffer {
        resources::index::create_buffer(self.device.as_ref().unwrap(), size)
    }

    pub fn make_buffer(
        device: &wgpu::Device,
        label: &str,
        size: wgpu::BufferAddress,
        buffer_usages: BufferUsages,
    ) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: buffer_usages,
            mapped_at_creation: false,
        })
    }
}
