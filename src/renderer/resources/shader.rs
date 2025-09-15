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
    F_WORLD => "functions/world.wgsl",
    F_BITWISE => "functions/bitwise.wgsl",
);

// voxel
include_shaders!(
    VOXEL_TYPES => "voxel/types.wgsl",
    VOXEL_CONST => "voxel/const.wgsl",
    VOXEL_CHUNK_MESH_ENTRY => "voxel/chunk_mesh_entry.wgsl",
    VOXEL_CHUNK_MESH_FACES => "voxel/chunk_mesh_faces.wgsl",
    VOXEL_CHUNK_MESH_VAO => "voxel/chunk_mesh_vao.wgsl",
);

pub fn main_shader() -> String {
    concat_shaders!(
        GLOBAL,
        VOXEL_TYPES,
        VOXEL_CONST,
        VOXEL_CHUNK_MESH_VAO,
        VERTEX_SHADER_ENTRY,
        FRAGMENT_SHADER_ENTRY,
        F_WORLD,
    )
}

pub fn chunk_meshing() -> String {
    concat_shaders!(
        GLOBAL,
        VOXEL_TYPES,
        VOXEL_CHUNK_MESH_ENTRY,
        VOXEL_CHUNK_MESH_FACES,
        VOXEL_CHUNK_MESH_VAO,
        F_WORLD,
        F_BITWISE,
    )
}

pub fn create(device: &wgpu::Device, source: Cow<str>) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("main_shader"),
        source: wgpu::ShaderSource::Wgsl(source),
    })
}
