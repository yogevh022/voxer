use std::borrow::Cow;
use wgpu::ShaderSource;
use crate::compute::geo::Plane;
use crate::renderer::gpu::{GPUVoxelChunk, GPUVoxelChunkAdjContent, GPUVoxelChunkContent, GPUVoxelFaceData, GPUDrawIndirectArgs, GPUChunkMeshEntry, GPUVoxelChunkHeader, GPUDispatchIndirectArgsAtomic, GPUPackedIndirectArgsAtomic};
use crate::renderer::gpu::vx_gpu_camera::VxGPUCamera;
use crate::world::{CHUNK_DIM, CHUNK_DIM_HALF};

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

include_shaders!(
    VX_SCREENSPACE => "vx/vx_screenspace.wgsl",
    VX_DEPTH_MIP_COMMON => "vx/vx_depth_mip_common.wgsl",
    VX_DEPTH_MIP_ONE_ENTRY => "vx/vx_depth_mip_one_entry.wgsl",
    VX_DEPTH_MIP_X_ENTRY => "vx/vx_depth_mip_x_entry.wgsl",
);

// general
include_shaders!(
    VERTEX_SHADER_ENTRY => "vert.wgsl",
    FRAGMENT_SHADER_ENTRY => "frag.wgsl",
);

// functions
include_shaders!(
    F_WORLD => "functions/world.wgsl",
    F_BITWISE => "functions/bitwise.wgsl",
    F_THREAD_MAPPING => "functions/thread_mapping.wgsl",
    F_MATH => "functions/math.wgsl",
    F_GEO => "functions/geo.wgsl",
    F_UNPACK_GPU_CHUNK_MESH_ENTRY => "functions/unpack_GPUChunkMeshEntry.wgsl",
    F_UNPACK_GPU_VOXEL_FACE_DATA => "functions/unpack_GPUVoxelFaceData.wgsl",
    F_MASK_INDEX => "functions/mask_index.wgsl",
);

// voxel
include_shaders!(
    VOXEL_CONST => "voxel/const.wgsl",
    VOXEL_CHUNK_MESH_ENTRY => "voxel/chunk_mesh_entry.wgsl",
    VOXEL_CHUNK_MESH_FACES => "voxel/chunk_mesh_faces.wgsl",
    VOXEL_CHUNK_MESH_VAO => "voxel/chunk_mesh_vao.wgsl",
    VOXEL_CHUNK_WRITE_ENTRY => "voxel/chunk_scattered_write.wgsl",
    VOXEL_CHUNK_CULL_ENTRY => "voxel/chunk_mdi_args.wgsl",
);

fn globals() -> String {
    let consts = include_shader_consts!(
        VOID_OFFSET: u32 = 1;
    );
    concat_shaders!(
        &consts,
        F_MASK_INDEX,
    )
}

pub const MAX_WORKGROUP_DIM_2D: u32 = 16;
pub const MAX_WORKGROUP_DIM_1D: u32 = MAX_WORKGROUP_DIM_2D * MAX_WORKGROUP_DIM_2D;

fn cfg_constants() -> String {
    include_shader_consts!(
        CFG_MAX_WORKGROUP_DIM_2D: u32 = MAX_WORKGROUP_DIM_2D;
        CFG_MAX_WORKGROUP_DIM_1D: u32 = MAX_WORKGROUP_DIM_1D;
        CFG_VAO_FACTOR: f32 = 0.35;
    )
}

fn geo_types() -> String {
    include_shader_types!(
        Plane
    )
}

fn meta_types() -> String {
    include_shader_types!(
        VxGPUCamera,
        GPUDrawIndirectArgs,
        GPUDispatchIndirectArgsAtomic,
        GPUPackedIndirectArgsAtomic,
    )
}

fn voxel_common() -> String {
    let consts = include_shader_consts!(
        CHUNK_DIM: u32 = CHUNK_DIM;
        CHUNK_DIM_HALF: u32 = CHUNK_DIM_HALF;
        CHUNK_BOUNDING_SPHERE_R: f32 = CHUNK_DIM_HALF as f32 * 1.75;
        INV_CHUNK_DIM: f32 = 1.0 / CHUNK_DIM as f32;
        INV_CHUNK_DIM_HALF: f32 = 1.0 / CHUNK_DIM_HALF as f32;
    );
    let types = include_shader_types!(
        GPUVoxelChunkContent,
        GPUVoxelChunkAdjContent,
        GPUVoxelChunk,
        GPUVoxelChunkHeader,
        GPUVoxelFaceData,
        GPUChunkMeshEntry,
    );
    concat_shaders!(&consts, &types)
}

pub fn render_wgsl() -> String {
    concat_shaders!(
        &cfg_constants(),
        &meta_types(),
        &geo_types(),
        &voxel_common(),
        VOXEL_CONST,
        VOXEL_CHUNK_MESH_VAO,
        VERTEX_SHADER_ENTRY,
        FRAGMENT_SHADER_ENTRY,
        F_BITWISE,
        F_UNPACK_GPU_VOXEL_FACE_DATA,
    )
}

pub fn chunk_meshing_wgsl() -> String {
    concat_shaders!(
        &cfg_constants(),
        &voxel_common(),
        &globals(),
        VOXEL_CHUNK_MESH_ENTRY,
        VOXEL_CHUNK_MESH_FACES,
        VOXEL_CHUNK_MESH_VAO,
        F_WORLD,
        F_BITWISE,
        F_UNPACK_GPU_CHUNK_MESH_ENTRY,
    )
}

pub fn chunk_mdi_args_wgsl() -> String {
    concat_shaders!(
        &cfg_constants(),
        &meta_types(),
        &geo_types(),
        &voxel_common(),
        &globals(),
        VX_SCREENSPACE,
        VOXEL_CHUNK_CULL_ENTRY,
        F_MATH,
        F_GEO,
        F_BITWISE,
        F_THREAD_MAPPING,
        F_UNPACK_GPU_CHUNK_MESH_ENTRY,
    )
}

pub fn chunk_write_wgsl() -> String {
    concat_shaders!(
        &cfg_constants(),
        &voxel_common(),
        F_THREAD_MAPPING,
        VOXEL_CHUNK_WRITE_ENTRY,
    )
}

pub fn depth_mip_one_wgsl() -> String {
    concat_shaders!(
        &cfg_constants(),
        VX_DEPTH_MIP_COMMON,
        VX_DEPTH_MIP_ONE_ENTRY,
    )
}

pub fn depth_mip_x_wgsl() -> String {
    concat_shaders!(
        &cfg_constants(),
        VX_DEPTH_MIP_COMMON,
        VX_DEPTH_MIP_X_ENTRY,
    )
}

// fixme better place for this
pub fn create_shader(device: &wgpu::Device, source: Cow<str>, label: &'static str) -> wgpu::ShaderModule {
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
