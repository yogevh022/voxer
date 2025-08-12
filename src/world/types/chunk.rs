use crate::compute::array::Array3D;
use crate::renderer::{Index, Vertex};
use crate::world::types::block::Block;
use bytemuck::{Pod, Zeroable};
use std::time::Instant;

pub const CHUNK_DIM: usize = 16;
pub const CHUNK_SLICE: usize = CHUNK_DIM * CHUNK_DIM;
pub const CHUNK_VOLUME: usize = CHUNK_SLICE * CHUNK_DIM;
pub const GPU_CHUNK_SIZE: usize = size_of::<GPUChunkEntryHeader>() + size_of::<ChunkBlocks>();
pub type ChunkBlocks = Array3D<Block, CHUNK_DIM>;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub(crate) last_visited: Option<Instant>,
    pub(crate) blocks: ChunkBlocks,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GPUChunkEntryHeader {
    pub exists: u32,
    pub vertex_offset: u32,
    pub index_offset: u32,
    pub vertex_count: u32,
    pub index_count: u32,
}

impl GPUChunkEntryHeader {
    pub fn new(
        vertex_offset: usize,
        index_offset: usize,
        vertex_count: u32,
        index_count: u32,
    ) -> Self {
        Self {
            exists: 1,
            vertex_offset: vertex_offset as u32,
            index_offset: index_offset as u32,
            vertex_count,
            index_count,
        }
    }
}
