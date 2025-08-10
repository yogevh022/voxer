use crate::render::renderer;
use crate::render::renderer::builder::RendererBuilder;
use std::borrow::Cow;

impl RendererBuilder<'_> {
    pub fn make_render_pipeline(
        &self,
        shader_source: Cow<str>,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::RenderPipeline {
        renderer::helpers::render_pipeline::create(
            self.device.as_ref().unwrap(),
            shader_source,
            bind_group_layouts,
            self.surface_format.unwrap(),
            "render_pipeline",
        )
    }
}
