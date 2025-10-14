use crate::world::types::{CHUNK_DIM, CHUNK_DIM_HALF, ChunkAdjacentBlocks, ChunkBlocks};
use bytemuck::{Pod, Zeroable, bytes_of};
use glam::IVec4;
use voxer_macros::ShaderType;

#[repr(C)]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPU4Bytes {
    pub data: u32,
}

#[repr(C, align(4))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUChunkMeshEntry {
    pub index: u32,
    pub face_count: u32,
    pub face_offset: u32,
}

impl GPUChunkMeshEntry {
    pub fn new(index: u32, face_count: u32, face_offset: u32) -> Self {
        Self {
            index,
            face_count,
            face_offset,
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

#[repr(C, align(16))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunkContent {
    blocks: [[[u32; CHUNK_DIM_HALF]; CHUNK_DIM]; CHUNK_DIM],
}

#[repr(C, align(16))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunkAdjContent {
    next_blocks: [[[u32; CHUNK_DIM_HALF]; CHUNK_DIM]; 3],
    prev_blocks: [[[u32; CHUNK_DIM_HALF]; CHUNK_DIM]; 3],
}

#[repr(C, align(16))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunk {
    // 11,280 bytes total
    pub position_index: IVec4,                // 16 bytes
    pub adj_content: GPUVoxelChunkAdjContent, // 3072 bytes
    pub content: GPUVoxelChunkContent,        // 8192 bytes
}

impl GPUVoxelChunk {
    pub fn new(position_index: IVec4, adj: &ChunkAdjacentBlocks, blocks: &ChunkBlocks) -> Self {
        // alignment safe transmutation
        // fixme implement this in a better way
        let gpu_content: GPUVoxelChunkContent = bytemuck::pod_read_unaligned(bytes_of(blocks));
        let gpu_adj_content: GPUVoxelChunkAdjContent = bytemuck::pod_read_unaligned(bytes_of(adj));
        Self {
            position_index,
            adj_content: gpu_adj_content,
            content: gpu_content,
        }
    }

    pub(crate) fn set_chunk_index(&mut self, chunk_index: u32) {
        // hack private function
        self.position_index.w = chunk_index as i32;
    }
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
