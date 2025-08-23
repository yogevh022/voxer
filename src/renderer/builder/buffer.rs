use super::{RendererBuilder, resources};
use wgpu::BufferUsages;

impl RendererBuilder<'_> {
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
