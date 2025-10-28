use crate::world::WorldConfig;
use crate::world::server::world::block::VoxelBlock;
use crate::world::server::world::chunk::VoxelChunk;
use crate::world::server::world::generation::VoxelChunkNoise;
use crate::world::server::world::{CHUNK_DIM, CHUNK_VOLUME, VoxelChunkBlocks, WorldGenerator};
use fastnoise2::SafeNode;
use fastnoise2::generator::{Generator, GeneratorWrapper, prelude};
use glam::IVec3;
use std::mem::MaybeUninit;
use std::ops::Add;

#[derive(Clone)]
pub struct EarthGen {
    config: WorldConfig,
}

impl WorldGenerator for EarthGen {
    fn noise(&self) -> GeneratorWrapper<SafeNode> {
        let ground_level = prelude::perlin()
            .domain_scale(0.02)
            .remap(-1.0, 1.0, -3.0, 3.0)
            .add(prelude::position_output([0.0, 1.0, 0.0, 0.0], [0.0; 4]).build());
        let ridges = prelude::perlin()
            .domain_scale(0.1)
            .ridged(2.0, 3.0, 3, 2.5)
            .remap(-1.0, 1.0, 1.0, 0.0)
            .powi(2)
            .remap(0.0, 1.0, -1.0, 2.0);

        ground_level.add(ridges.build()).build()
    }
    fn chunk(&self, position: IVec3) -> VoxelChunk {
        let noise = self.chunk_noise(position);
        let (voxel_count, voxels) = self.chunk_voxels(noise);
        VoxelChunk::new(position, voxels, voxel_count)
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

    fn chunk_voxels(&self, voxel_chunk_noise: VoxelChunkNoise) -> (u32, VoxelChunkBlocks) {
        let mut blocks: [MaybeUninit<VoxelBlock>; CHUNK_VOLUME] =
            [MaybeUninit::uninit(); CHUNK_VOLUME];
        let out = unsafe { &mut *(blocks.as_mut_ptr() as *mut VoxelChunkBlocks) };

        let mut voxel_count = 0u32;
        for z in 0..CHUNK_DIM {
            for y in 0..CHUNK_DIM {
                for x in 0..CHUNK_DIM {
                    let density = voxel_chunk_noise[z][y][x];
                    let voxel = Self::voxel_from_noise(density);
                    if !voxel.is_transparent() {
                        voxel_count += 1;
                    }
                    out[z][y][x] = voxel;
                }
            }
        }

        // SAFETY: all blocks are initialized above
        let voxel_chunk_blocks = unsafe { std::mem::transmute(blocks) };
        (voxel_count, voxel_chunk_blocks)
    }

    fn voxel_from_noise(density: f32) -> VoxelBlock {
        match density {
            n if n < 0.0 => VoxelBlock { value: 1u16 << 15 },
            _ => VoxelBlock::EMPTY,
        }
    }
}
