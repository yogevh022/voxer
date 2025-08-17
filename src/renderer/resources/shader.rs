use std::borrow::Cow;

const SHADER_TYPES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/renderer/shaders/types.wgsl"
));

const VERT_SHADER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/renderer/shaders/vert.wgsl"
));
const FRAG_SHADER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/renderer/shaders/frag.wgsl"
));

const MESHGEN_SHADER: &str = concat!(
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/renderer/shaders/chunk_meshing/entry.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/renderer/shaders/chunk_meshing/faces.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/renderer/shaders/chunk_meshing/quads.wgsl"
    )),
);

pub fn main_shader_source() -> String {
    format!("{}\n{}", VERT_SHADER, FRAG_SHADER)
}

pub fn meshgen_shader_source() -> String {
    format!("{}\n{}", SHADER_TYPES, MESHGEN_SHADER)
}

pub fn create(device: &wgpu::Device, source: Cow<str>) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("main_shader"),
        source: wgpu::ShaderSource::Wgsl(source),
    })
}
