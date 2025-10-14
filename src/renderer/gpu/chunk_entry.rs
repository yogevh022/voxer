use crate::world::types::{CHUNK_DIM, CHUNK_DIM_HALF, ChunkAdjacentBlocks, ChunkBlocks};
use bytemuck::{Pod, Zeroable, bytes_of};
use glam::{IVec3, IVec4};
use std::mem::{MaybeUninit, size_of};
use voxer_macros::ShaderType;
use wgpu::wgt::DrawIndirectArgs;

#[repr(C)]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPU4Bytes {
    pub data: u32,
}

#[repr(C, align(16))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUChunkMeshEntry {
    pub index: u32,
    pub face_count: u32,
    pub face_offset: u32,
    _padding: u32,
}

impl GPUChunkMeshEntry {
    pub fn new(index: u32, face_count: u32, face_offset: u32) -> Self {
        Self {
            index,
            face_count,
            face_offset,
            _padding: 0,
        }
    }
}

#[repr(C, align(16))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUDrawIndirectArgs {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
}

#[repr(C, align(32))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunkContent {
    blocks: [[[u32; CHUNK_DIM_HALF]; CHUNK_DIM]; CHUNK_DIM],
}

#[repr(C, align(32))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunkAdjContent {
    next_blocks: [[[u32; CHUNK_DIM_HALF]; CHUNK_DIM]; 3],
    prev_blocks: [[[u32; CHUNK_DIM_HALF]; CHUNK_DIM]; 3],
}

#[repr(C, align(4))]
#[derive(ShaderType)]
pub struct GPUVoxelFaceData {
    word_a: u32,
    // world_x: 24b
    // top_left_R: 6b
    // top_left_AO: 2b
    word_b: u32,
    // world_z: 24b
    // top_right_R: 6b
    // top_right_AO: 2b
    word_c: u32,
    // world_y: 12b
    // bot_left_R: 6b
    // bot_left_G: 6b
    // bot_left_B: 6b
    // bot_left_AO: 2b
    word_d: u32,
    // bot_right_R: 6b
    // bot_right_G: 6b
    // bot_right_B: 6b
    // top_left_G: 6b
    // top_left_B: 6b
    // bot_right_AO: 2b
    word_e: u32,
    // voxel: 16b
    // top_right_G: 6b
    // top_right_B: 6b
    // face_id: 3b
    // 1b free

    // total: 20 bytes
}

#[repr(C, align(8))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunkBufferData {
    pub offset: u32,
    pub face_count: u32,
}

#[repr(C, align(32))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunkHeader {
    pub buffer_data: GPUVoxelChunkBufferData,
    pub slab_index: u32,
    _padding: u32,
    pub position: IVec3,
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

#[repr(C, align(32))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunk {
    pub header: GPUVoxelChunkHeader,          // 32 bytes
    pub adj_content: GPUVoxelChunkAdjContent, // 3072 bytes
    pub content: GPUVoxelChunkContent,        // 8192 bytes
                                              // 11296 bytes total
}

impl GPUVoxelChunk {
    pub fn new(
        header: GPUVoxelChunkHeader,
        adj: &ChunkAdjacentBlocks,
        blocks: &ChunkBlocks,
    ) -> Self {
        // alignment safe transmutation
        // fixme implement this in a better way
        let gpu_content: GPUVoxelChunkContent = bytemuck::pod_read_unaligned(bytes_of(blocks));
        let gpu_adj_content: GPUVoxelChunkAdjContent = bytemuck::pod_read_unaligned(bytes_of(adj));
        Self {
            header,
            adj_content: gpu_adj_content,
            content: gpu_content,
        }
    }
}
