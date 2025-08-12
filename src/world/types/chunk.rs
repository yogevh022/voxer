use crate::compute::array::Array3D;
use crate::render::types::{Index, Mesh, Vertex};
use crate::world::types::block::Block;
use std::time::Instant;

pub const CHUNK_DIM: usize = 16;
pub const CHUNK_SLICE: usize = CHUNK_DIM * CHUNK_DIM;
pub const CHUNK_VOLUME: usize = CHUNK_SLICE * CHUNK_DIM;
pub type ChunkBlocks = Array3D<Block, CHUNK_DIM>;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub(crate) last_visited: Option<Instant>,
    pub(crate) blocks: ChunkBlocks,
    pub(crate) mesh: Option<Mesh>,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct GPUChunkEntry<'a> {
    pub exists: u32,
    pub vertex_offset: u32,
    pub index_offset: u32,
    pub vertex_count: u32,
    pub index_count: u32,
    pub blocks: &'a ChunkBlocks,
}

impl<'a> GPUChunkEntry<'a> {
    pub fn new(
        blocks: &'a ChunkBlocks,
        vertex_offset: usize,
        index_offset: usize,
        face_count: usize,
    ) -> Self {
        Self {
            exists: 1,
            vertex_offset: vertex_offset as u32,
            index_offset: index_offset as u32,
            vertex_count: (face_count * 4 * size_of::<Vertex>()) as u32,
            index_count: (face_count * 6 * size_of::<Index>()) as u32,
            blocks,
        }
    }
}
