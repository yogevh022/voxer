use crate::compute::array::Array3D;
use crate::world::types::block::VoxelBlock;
use glam::IVec3;
use crate::compute;

pub const CHUNK_DIM: usize = 16;
pub const PACKED_CHUNK_DIM: usize = 8;
pub const CHUNK_SLICE: usize = CHUNK_DIM * CHUNK_DIM;
pub const CHUNK_VOLUME: usize = CHUNK_SLICE * CHUNK_DIM;
pub type ChunkBlocks = Array3D<VoxelBlock, CHUNK_DIM, CHUNK_DIM, CHUNK_DIM>;
pub type ChunkAdjacentBlocks = Array3D<VoxelBlock, 3, CHUNK_DIM, CHUNK_DIM>;


#[derive(Debug, Clone)]
pub struct Chunk {
    pub position: IVec3,
    pub blocks: ChunkBlocks,
    pub face_count: Option<usize>,
    pub adjacent_blocks: ChunkAdjacentBlocks,
    pub solid_count: usize,
}

impl Chunk {
    pub fn new(position: IVec3, blocks: ChunkBlocks, solid_count: usize) -> Self {
        Self {
            position,
            blocks,
            face_count: None,
            adjacent_blocks: ChunkAdjacentBlocks::default(),
            solid_count,
        }
    }
}