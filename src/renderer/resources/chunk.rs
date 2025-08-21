use crate::renderer::gpu::GPUChunkEntry;

pub fn create_chunk_buffer(device: &wgpu::Device, size: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("chunk_buffer"),
        size: size as u64,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
