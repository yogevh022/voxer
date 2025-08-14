use crate::world::types::{CHUNK_DIM, ChunkBlocks, PACKED_CHUNK_DIM};
use encase::ShaderType;
use glam::Vec3;

pub const GPU_CHUNK_SIZE: usize = size_of::<GPUChunkEntry>();
type GPUPackedBlockPair = u32;
type GPUChunkBlocks = [[[GPUPackedBlockPair; PACKED_CHUNK_DIM]; CHUNK_DIM]; CHUNK_DIM];

#[derive(ShaderType)]
pub struct GPUChunkEntry {
    pub header: GPUChunkEntryHeader,
    pub blocks: GPUChunkBlocks,
}

impl GPUChunkEntry {
    pub fn new(header: GPUChunkEntryHeader, blocks: ChunkBlocks) -> Self {
        let gpu_blocks: GPUChunkBlocks = unsafe { std::mem::transmute(blocks) };
        Self {
            header,
            blocks: gpu_blocks,
        }
    }
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct GPUChunkEntryHeader {
    pub vertex_allocation: u32,
    pub index_allocation: u32,
    pub vertex_count: u32,
    pub index_count: u32,
    pub slab_index: u32,
    pub world_position: Vec3,
}

impl GPUChunkEntryHeader {
    pub fn new(
        vertex_allocation: u32,
        index_allocation: u32,
        vertex_count: u32,
        index_count: u32,
        slab_index: u32,
        world_position: Vec3,
    ) -> Self {
        Self {
            vertex_allocation,
            index_allocation,
            vertex_count,
            index_count,
            slab_index,
            world_position,
        }
    }
}
