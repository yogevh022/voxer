use std::borrow::Cow;
use wgpu::ShaderSource;
use crate::app::app_renderer::UniformCameraView;
use crate::compute::geo::Plane;
use crate::renderer::gpu::{GPUVoxelChunk, GPUVoxelChunkAdjContent, GPUVoxelChunkContent, GPUVoxelFaceData, GPUDrawIndirectArgs, GPUChunkMeshEntry};
use crate::world::types::{CHUNK_DIM, CHUNK_DIM_HALF};

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

macro_rules! include_shader_types {
    ($($shader_type:ty),* $(,)?) => {
        {
            let mut shader_types = String::new();
            $(
                let shader_src: &'static str = <$shader_type>::SHADER_SOURCE;
                shader_types.push_str(shader_src);
                shader_types.push('\n');
            )*
            shader_types
        }
    }
}

macro_rules! include_shader_consts {
    ( $( $name:ident: $ty:ty = $value:expr );* $(;)? ) => {
        {
            let mut shader_consts = String::new();
            $(
                shader_consts.push_str("const ");
                shader_consts.push_str(stringify!($name));
                shader_consts.push_str(": ");
                shader_consts.push_str(stringify!($ty));
                shader_consts.push_str(" = ");
                let value_str = $value.to_string();
                shader_consts.push_str(&value_str);
                shader_consts.push_str(";\n");
            )*
            shader_consts
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
    VOXEL_CONST => "voxel/const.wgsl",
    VOXEL_CHUNK_MESH_ENTRY => "voxel/chunk_mesh_entry.wgsl",
    VOXEL_CHUNK_MESH_FACES => "voxel/chunk_mesh_faces.wgsl",
    VOXEL_CHUNK_MESH_VAO => "voxel/chunk_mesh_vao.wgsl",
    VOXEL_CHUNK_WRITE_ENTRY => "voxel/chunk_write.wgsl",
    VOXEL_CHUNK_CULL_ENTRY => "voxel/chunk_cull.wgsl",
);

fn voxel_common() -> (String, String) {
    let consts = include_shader_consts!(
        CHUNK_DIM: u32 = CHUNK_DIM;
        CHUNK_DIM_HALF: u32 = CHUNK_DIM_HALF;
        MAX_WORKGROUP_DIM_2D: u32 = 16;
        CFG_VAO_FACTOR: f32 = 0.35;
    );
    let types = include_shader_types!(
        GPUVoxelChunkContent,
        GPUVoxelChunkAdjContent,
        GPUVoxelChunk,
        GPUVoxelFaceData,

        GPUChunkMeshEntry,
        GPUDrawIndirectArgs,
        Plane
    );
    (consts, types)
}

pub fn main_shader() -> String {
    let uniform_camera_view = include_shader_types!(
        UniformCameraView
    );
    let (consts, types) = voxel_common();
    concat_shaders!(
        GLOBAL,
        &uniform_camera_view,
        &consts,
        &types,
        VOXEL_CONST,
        VOXEL_CHUNK_MESH_VAO,
        VERTEX_SHADER_ENTRY,
        FRAGMENT_SHADER_ENTRY,
        F_BITWISE,
    )
}

pub fn chunk_meshing() -> String {
    let (consts, types) = voxel_common();
    concat_shaders!(
        GLOBAL,
        &consts,
        &types,
        VOXEL_CHUNK_MESH_ENTRY,
        VOXEL_CHUNK_MESH_FACES,
        VOXEL_CHUNK_MESH_VAO,
        F_WORLD,
        F_BITWISE,
    )
}

pub fn chunk_draw_args() -> String {
    let uniform_camera_view = include_shader_types!(
        UniformCameraView
    );
    let (consts, types) = voxel_common();
    concat_shaders!(
        GLOBAL,
        &consts,
        &types,
        &uniform_camera_view,
        F_BITWISE,
        VOXEL_CHUNK_CULL_ENTRY,
    )
}

pub fn chunk_write() -> String {
    let (consts, types) = voxel_common();
    concat_shaders!(
        GLOBAL,
        &consts,
        &types,
        VOXEL_CHUNK_WRITE_ENTRY,
    )
}

pub fn create(device: &wgpu::Device, source: Cow<str>, label: &'static str) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: ShaderSource::Wgsl(source),
    })
}

pub struct VxShaderTypeData {
    pub(crate) name: &'static str,
    pub(crate) stride: usize,
}

pub trait ShaderType {
    const SHADER_SOURCE: &'static str;
    const SHADER_TYPE_DATA: VxShaderTypeData;
}
