use crate::compute::array::Array3D;
use crate::world::types::block::Block;
use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use std::time::Instant;

pub const CHUNK_DIM: usize = 16;
pub const CHUNK_SLICE: usize = CHUNK_DIM * CHUNK_DIM;
pub const CHUNK_VOLUME: usize = CHUNK_SLICE * CHUNK_DIM;
pub type ChunkBlocks = Array3D<Block, CHUNK_DIM>;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub(crate) last_visited: Option<Instant>,
    pub(crate) blocks: ChunkBlocks,
}
