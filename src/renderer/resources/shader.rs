use std::borrow::Cow;

const VERT_SHADER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/renderer/shaders/vert.wgsl"
));
const FRAG_SHADER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/renderer/shaders/frag.wgsl"
));

const MESHGEN_SHADER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/renderer/shaders/meshgen.wgsl"
));

pub fn main_shader_source() -> String {
    format!("{}\n{}", VERT_SHADER, FRAG_SHADER)
}

pub fn meshgen_shader_source() -> &'static str {
    MESHGEN_SHADER
}

pub fn create(device: &wgpu::Device, source: Cow<str>) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("main_shader"),
        source: wgpu::ShaderSource::Wgsl(source),
    })
}
