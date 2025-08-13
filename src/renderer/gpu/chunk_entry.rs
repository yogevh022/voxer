use crate::world::types::ChunkBlocks;
use bytemuck::{Pod, Zeroable};
use glam::Vec3;

pub const GPU_CHUNK_SIZE: usize = size_of::<GPUChunkEntryHeader>() + size_of::<ChunkBlocks>() + 12;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GPUChunkEntryHeader {
    pub vertex_allocation: u32,
    pub index_allocation: u32,
    pub vertex_count: u32,
    pub index_count: u32,
    pub world_position: Vec3,
}

impl GPUChunkEntryHeader {
    pub fn new(
        vertex_allocation: u32,
        index_allocation: u32,
        vertex_count: u32,
        index_count: u32,
        world_position: Vec3,
    ) -> Self {
        Self {
            vertex_allocation,
            index_allocation,
            vertex_count,
            index_count,
            world_position,
        }
    }
}
