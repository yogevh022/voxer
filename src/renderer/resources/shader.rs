use std::borrow::Cow;
macro_rules! include_shaders {
    ($($name:ident => $file:literal), * $(,)?) => (
        $(
            pub const $name: &str = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/renderer/shaders/",
                $file
            ));
        )*
    )
}

macro_rules! concat_shaders {
    ($($shader:expr),* $(,)?) => {
        {
            let mut result = String::new();
            $(
                result.push_str($shader);
                result.push('\n');
            )*
            result
        }
    };
}

// general
include_shaders!(
    GLOBAL => "global.wgsl",
    VERTEX_SHADER_ENTRY => "vert.wgsl",
    FRAGMENT_SHADER_ENTRY => "frag.wgsl",
);

// functions
include_shaders!(
    F_TRANSFORM => "functions/transform.wgsl",
    F_WORLD => "functions/world.wgsl",
    F_BITWISE => "functions/bitwise.wgsl",
);

// meshing
include_shaders!(
    CHUNK_MESHING_ENTRY => "chunk_meshing/entry.wgsl",
    CHUNK_MESHING_VAO => "chunk_meshing/vao.wgsl",
    // CHUNK_MESHING_QUADS => "chunk_meshing/quads.wgsl",
    CHUNK_MESHING_FACES => "chunk_meshing/faces.wgsl",
    CHUNK_MESHING_TYPES => "chunk_meshing/types.wgsl",
);

pub fn main_shader() -> String {
    concat_shaders!(VERTEX_SHADER_ENTRY, FRAGMENT_SHADER_ENTRY)
}

pub fn chunk_meshing() -> String {
    concat_shaders!(
        GLOBAL,
        F_TRANSFORM,
        F_WORLD,
        F_BITWISE,
        CHUNK_MESHING_TYPES,
        CHUNK_MESHING_VAO,
        CHUNK_MESHING_ENTRY,
        // CHUNK_MESHING_QUADS,
        CHUNK_MESHING_FACES
    )
}

pub fn create(device: &wgpu::Device, source: Cow<str>) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("main_shader"),
        source: wgpu::ShaderSource::Wgsl(source),
    })
}
