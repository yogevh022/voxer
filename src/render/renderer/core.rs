use crate::render::renderer::builder::RendererBuilder;

pub(crate) struct Renderer<'window> {
    pub(crate) surface: wgpu::Surface<'window>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
}

impl<'window> From<RendererBuilder<'window>> for Renderer<'window> {
    fn from(value: RendererBuilder<'window>) -> Self {
        Self {
            surface: value.surface.unwrap(),
            device: value.device.unwrap(),
            queue: value.queue.unwrap(),
            config: value.config.unwrap(),
        }
    }
}

impl Renderer<'_> {
    pub fn write_buffer(
        &mut self,
        buffer: &wgpu::Buffer,
        offset: wgpu::BufferAddress,
        data: &[u8],
    ) {
        self.queue.write_buffer(buffer, offset, data)
    }
}
