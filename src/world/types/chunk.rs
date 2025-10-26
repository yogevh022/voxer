use crate::compute::array::Array3D;
use crate::world::types::block::VoxelBlock;
use glam::IVec3;
use crate::compute;
use crate::compute::chunk::VoxelChunkMeshMeta;

pub const CHUNK_DIM: usize = 16;
pub const CHUNK_DIM_HALF: usize = CHUNK_DIM / 2;
pub const INV_CHUNK_DIM: f32 = 1.0 / CHUNK_DIM as f32;
pub const INV_CHUNK_DIM_HALF: f32 = 1.0 / CHUNK_DIM_HALF as f32;
pub const CHUNK_SLICE: usize = CHUNK_DIM * CHUNK_DIM;
pub const CHUNK_VOLUME: usize = CHUNK_SLICE * CHUNK_DIM;
pub type ChunkBlocks = Array3D<VoxelBlock, CHUNK_DIM, CHUNK_DIM, CHUNK_DIM>;
pub type ChunkAdjBlocks = Array3D<VoxelBlock, 6, CHUNK_DIM, CHUNK_DIM>; // px py pz mx my mz


#[derive(Debug, Clone)]
pub struct Chunk {
    pub position: IVec3,
    pub blocks: ChunkBlocks,
    pub mesh_meta: Option<VoxelChunkMeshMeta>,
    pub adjacent_blocks: ChunkAdjBlocks,
    pub solid_count: usize,
}

impl Chunk {
    pub fn new(position: IVec3, blocks: ChunkBlocks, solid_count: usize) -> Self {
        Self {
            position,
            blocks,
            mesh_meta: None,
            adjacent_blocks: ChunkAdjBlocks::default(),
            solid_count,
        }
    }
}