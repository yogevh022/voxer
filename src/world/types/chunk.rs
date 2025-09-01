use crate::compute::array::Array3D;
use crate::world::types::block::Block;
use glam::IVec3;
use crate::compute;

pub const CHUNK_DIM: usize = 16;
pub const PACKED_CHUNK_DIM: usize = 8;
pub const CHUNK_SLICE: usize = CHUNK_DIM * CHUNK_DIM;
pub const CHUNK_VOLUME: usize = CHUNK_SLICE * CHUNK_DIM;
pub type ChunkBlocks = Array3D<Block, CHUNK_DIM, CHUNK_DIM, CHUNK_DIM>;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub(crate) position: IVec3,
    pub(crate) blocks: ChunkBlocks,
    pub(crate) solid_count: usize,
}

impl Chunk {
    pub fn new(position: IVec3, blocks: ChunkBlocks, solid_count: usize) -> Self {
        Self {
            position,
            blocks,
            solid_count,
        }
    }
}