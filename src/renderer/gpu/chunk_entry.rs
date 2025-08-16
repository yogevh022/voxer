use crate::world::types::{CHUNK_DIM, ChunkBlocks, PACKED_CHUNK_DIM};
use bytemuck::{Pod, Zeroable};
use encase::ShaderType;
use glam::Vec3;

type GPUPackedBlockPair = u32;
type GPUChunkBlocks = [[[GPUPackedBlockPair; PACKED_CHUNK_DIM]; CHUNK_DIM]; CHUNK_DIM];

#[repr(C)]
#[derive(Clone, Copy, Debug, ShaderType, Pod, Zeroable)]
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

    pub const fn size() -> usize {
        size_of::<GPUChunkEntry>() + 16
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable, ShaderType)]
pub struct GPUChunkEntryHeader {
    pub vertex_offset: u32, // in position not bytes
    pub index_offset: u32, // in position not bytes
    pub vertex_count: u32,
    pub index_count: u32,
    pub slab_index: u32,
    pub world_position: Vec3, // 32
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
            vertex_offset: vertex_allocation,
            index_offset: index_allocation,
            vertex_count,
            index_count,
            slab_index,
            world_position,
        }
    }
}
