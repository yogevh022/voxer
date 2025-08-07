use crate::render::renderer::helpers;
use std::borrow::Cow;

pub fn create(
    device: &wgpu::Device,
    shader_source: Cow<str>,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    label: &str,
) -> wgpu::ComputePipeline {
    let shader = helpers::shader::create(&device, shader_source);
    helpers::pipeline::create_compute(&device, bind_group_layouts, &shader, label)
}
