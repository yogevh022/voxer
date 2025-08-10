use crate::render::renderer;
use crate::render::renderer::builder::RendererBuilder;

impl RendererBuilder<'_> {
    pub fn make_vertex_buffer(&self, size: u64) -> wgpu::Buffer {
        renderer::helpers::vertex::create_buffer(self.device.as_ref().unwrap(), size)
    }

    pub fn make_index_buffer(&self, size: u64) -> wgpu::Buffer {
        renderer::helpers::index::create_buffer(self.device.as_ref().unwrap(), size)
    }
}
