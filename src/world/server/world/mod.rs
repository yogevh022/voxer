mod earth;
pub mod generation;
mod earth_gen;
pub mod chunk;
pub mod block;

use fastnoise2::generator::GeneratorWrapper;
use fastnoise2::SafeNode;
use glam::{IVec3, USizeVec3};
pub use earth::Earth;
use crate::world::server::world::block::VoxelBlock;
use crate::world::server::world::chunk::VoxelChunk;

pub const CHUNK_DIM: usize = 16;
pub const CHUNK_DIM_HALF: usize = CHUNK_DIM / 2;
pub const CHUNK_VOLUME: usize = CHUNK_DIM * CHUNK_DIM * CHUNK_DIM;

pub type VoxelChunkBlocks = [[[VoxelBlock; CHUNK_DIM]; CHUNK_DIM]; CHUNK_DIM];
pub type VoxelChunkAdjBlocks = [[[VoxelBlock; CHUNK_DIM]; CHUNK_DIM]; 6]; // px py pz mx my mz


#[derive(Clone, Copy, Debug)]
pub struct WorldConfig {
    pub seed: i32,
    pub noise_scale: f64,
    pub max_world_size: USizeVec3,
}

pub trait World {
    fn tick(&mut self);
    fn request_chunks(&mut self, positions: &[IVec3]) -> Vec<&VoxelChunk>;
    fn request_chunk_generation(&mut self);
    fn start_simulation(&mut self);
    fn stop_simulation(&mut self);
}

pub trait WorldGenerator: Clone + Send + Sync + 'static {
    fn noise(&self) -> GeneratorWrapper<SafeNode>;
    fn chunk(&self, position: IVec3) -> VoxelChunk;
}