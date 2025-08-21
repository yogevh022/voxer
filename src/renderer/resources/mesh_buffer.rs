use crate::renderer::resources;

pub struct MeshBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl MeshBuffer {
    pub fn new(device: &wgpu::Device, vertex_size: usize, index_size: usize) -> Self {
        Self {
            vertex_buffer: resources::vertex::create_buffer(device, vertex_size as u64),
            index_buffer: resources::index::create_buffer(device, index_size as u64),
        }
    }
}
