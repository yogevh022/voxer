use fastnoise2::generator::{Generator, GeneratorWrapper};
use glam::IVec3;
use std::mem::MaybeUninit;
use fastnoise2::SafeNode;
use crate::world::server::world::chunk::VoxelChunk;
use crate::world::server::world::generation::VoxelChunkNoise;
use crate::world::server::world::{VoxelChunkBlocks, WorldGenerator, CHUNK_DIM, CHUNK_VOLUME};
use crate::world::server::world::block::VoxelBlock;
use crate::world::WorldConfig;

#[derive(Clone)]
pub struct EarthGen {
    config: WorldConfig,
}

impl WorldGenerator for EarthGen {
    fn noise(&self) -> GeneratorWrapper<SafeNode> {
        fastnoise2::generator::prelude::opensimplex2().build()
    }
    fn chunk(&self, position: IVec3) -> VoxelChunk {
        let noise = self.chunk_noise(position);
        let voxels = self.chunk_voxels(noise, position);
        VoxelChunk::new(position, voxels)
    }
}

impl EarthGen {
    pub fn new(config: WorldConfig) -> Self {
        Self { config }
    }

    fn chunk_noise(&self, position: IVec3) -> VoxelChunkNoise {
        let mut noise_out: [MaybeUninit<f32>; CHUNK_VOLUME] = [MaybeUninit::uninit(); CHUNK_VOLUME];
        let out = unsafe { &mut *(noise_out.as_mut_ptr() as *mut [f32; CHUNK_VOLUME]) };

        let start = position * CHUNK_DIM as i32;
        let dim = CHUNK_DIM as i32;
        let scale = self.config.noise_scale as f32;
        let seed = self.config.seed;
        let noise = self.noise();
        noise.gen_uniform_grid_3d(out, start.z, start.y, start.x, dim, dim, dim, scale, seed);

        // SAFETY: certainly initialized after noise generation
        unsafe { std::mem::transmute(noise_out) }
    }

    fn chunk_voxels(&self, voxel_chunk_noise: VoxelChunkNoise, position: IVec3) -> VoxelChunkBlocks {
        let mut blocks: [MaybeUninit<VoxelBlock>; CHUNK_VOLUME] = [MaybeUninit::uninit(); CHUNK_VOLUME];
        let out = unsafe { &mut *(blocks.as_mut_ptr() as *mut VoxelChunkBlocks) };

        let ground_level = 0.0;
        let amplitude = 10.0;

        let chunk_world_y = position.y as f32 * CHUNK_DIM as f32;
        for z in 0..CHUNK_DIM {
            for y in 0..CHUNK_DIM {
                let world_y = chunk_world_y + y as f32;
                for x in 0..CHUNK_DIM {
                    let noise_value = voxel_chunk_noise[z][y][x];
                    let density = (world_y - ground_level) + (noise_value * amplitude);
                    out[z][y][x] = Self::voxel_from_noise(density);
                }
            }
        }

        // SAFETY: all blocks are initialized above
        unsafe { std::mem::transmute(blocks) }
    }

    fn voxel_from_noise(density: f32) -> VoxelBlock {
        match density {
            n if n < 0.0 => VoxelBlock { value: 1u16 << 15 },
            _ => VoxelBlock::EMPTY,
        }
    }
}