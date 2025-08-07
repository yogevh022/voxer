use crate::render::renderer::helpers;
use std::borrow::Cow;

pub fn create(
    device: &wgpu::Device,
    shader_source: Cow<str>,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    surface_format: wgpu::TextureFormat,
    label: &str,
) -> wgpu::RenderPipeline {
    let shader = helpers::shader::create(&device, shader_source);
    helpers::pipeline::create_render(&device, bind_group_layouts, &shader, surface_format, label)
}
