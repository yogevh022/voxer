use super::{RendererBuilder, resources};
use std::borrow::Cow;

impl RendererBuilder<'_> {
    pub fn make_render_pipeline(
        &self,
        shader_source: Cow<str>,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::RenderPipeline {
        let shader = resources::shader::create(self.device.as_ref().unwrap(), shader_source);
        resources::pipeline::create_render(
            self.device.as_ref().unwrap(),
            bind_group_layouts,
            &shader,
            self.surface_format.unwrap(),
            "render_pipeline",
        )
    }
}
