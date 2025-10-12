use crate::world::types::{CHUNK_DIM, CHUNK_DIM_HALF, ChunkBlocks};
use bytemuck::{Pod, Zeroable};
use glam::IVec3;
use voxer_macros::ShaderType;
use wgpu::wgt::DrawIndirectArgs;

#[repr(C)]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPU4Bytes {
    pub data: u32,
}

#[repr(C)]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUDrawIndirectArgs {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
}

#[repr(C, align(8))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunkContent {
    blocks: [[[u32; CHUNK_DIM_HALF]; CHUNK_DIM]; CHUNK_DIM],
}

#[repr(C, align(8))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunkAdjContent {
    next_blocks: [[[u32; CHUNK_DIM_HALF]; CHUNK_DIM]; 3],
    prev_blocks: [[[u32; CHUNK_DIM_HALF]; CHUNK_DIM]; 3],
}

#[repr(C, align(8))]
#[derive(ShaderType)]
pub struct GPUVoxelFaceData {
    position_fid_illum_ocl: u32,
    // position 12b
    // face id 3b
    // illumination 5b
    // occlusion count 8b
    // 1b free
    ypos_voxel: u32,
    // y pos i16 16b
    // voxel_type 16b
}

#[repr(C, align(8))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunkBufferData {
    pub offset: u32,
    pub face_count: u32,
}

#[repr(C, align(16))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunkHeader {
    pub position: IVec3,
    pub slab_index: u32,
    pub buffer_data: GPUVoxelChunkBufferData,
    _padding: u32,
    _cpu_padding: u32,
}

impl GPUVoxelChunkHeader {
    pub fn new(offset: u32, face_count: u32, slab_index: u32, position: IVec3) -> Self {
        let buffer_data = GPUVoxelChunkBufferData { face_count, offset };
        Self {
            buffer_data,
            slab_index,
            position,
            _padding: 0,
            _cpu_padding: 0,
        }
    }

    pub fn draw_indirect_args(&self) -> DrawIndirectArgs {
        let packed_xz = self.position.x as u16 as u32 | ((self.position.z as u16 as u32) << 16);
        DrawIndirectArgs {
            vertex_count: self.buffer_data.face_count * 6,
            instance_count: 1,
            first_vertex: self.buffer_data.offset.wrapping_mul(6),
            first_instance: packed_xz,
        }
    }
}

#[repr(C, align(16))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunk {
    pub header: GPUVoxelChunkHeader,
    pub adj_content: GPUVoxelChunkAdjContent,
    pub content: GPUVoxelChunkContent,
}

impl GPUVoxelChunk {
    pub fn new(
        header: GPUVoxelChunkHeader,
        adj: GPUVoxelChunkAdjContent,
        blocks: ChunkBlocks,
    ) -> Self {
        let gpu_chunk_content: GPUVoxelChunkContent = unsafe { std::mem::transmute(blocks) };
        Self {
            header,
            adj_content: adj,
            content: gpu_chunk_content,
        }
    }
}
