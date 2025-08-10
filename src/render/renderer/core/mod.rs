use crate::render::renderer::builder::RendererBuilder;

mod chunks;
mod render;

pub(crate) struct Renderer<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
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
