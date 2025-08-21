use super::{RendererBuilder, resources};
use std::borrow::Cow;

impl RendererBuilder<'_> {
    pub fn make_render_pipeline(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        shader_source: Cow<str>,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::RenderPipeline {
        let shader = resources::shader::create(device, shader_source);
        resources::pipeline::create_render(
            device,
            bind_group_layouts,
            &shader,
            surface_format,
            "render_pipeline",
        )
    }
}
