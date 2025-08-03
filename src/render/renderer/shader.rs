use crate::{FRAG_SHADER, VERT_SHADER};
use std::borrow::Cow;

pub fn main_shader_source() -> String {
    format!("{}\n{}", VERT_SHADER, FRAG_SHADER)
}

pub fn create(device: &wgpu::Device, source: Cow<str>) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("main_shader"),
        source: wgpu::ShaderSource::Wgsl(source),
    })
}
