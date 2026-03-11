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
        // fixme temp
        if position == IVec3::new(0, 2, 0) {
            return test_house_chunk(position);
        }

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

// fixme temp
fn test_house_chunk(position: IVec3) -> VoxelChunk {
    let mut blocks: VoxelChunkBlocks = [[[VoxelBlock::EMPTY; CHUNK_DIM]; CHUNK_DIM]; CHUNK_DIM];

    let solid_block = VoxelBlock { value: 1u16 << 15 };

    let floor_height = 2;
    let wall_height = 3;

    // floor
    for z in 0..CHUNK_DIM {
        for y in 0..=floor_height {
            for x in 0..CHUNK_DIM {
                blocks[z][y][x] = solid_block;
            }
        }
    }

    // walls
    for wall_y in (floor_height + 1)..(floor_height + 1 + wall_height) {
        for wall_x in 2..CHUNK_DIM - 2 {
            blocks[2][wall_y][wall_x] = solid_block;
            blocks[CHUNK_DIM - 2 - 1][wall_y][wall_x] = solid_block;
        }
        for wall_z in 2..CHUNK_DIM - 2 {
            blocks[wall_z][wall_y][2] = solid_block;
            blocks[wall_z][wall_y][CHUNK_DIM - 2 - 1] = solid_block;
        }
    }
    for door_y_delta in 0..2 {
        let y = floor_height + 1 + door_y_delta;
        for door_x_delta in 0..2 {
            let x = (CHUNK_DIM / 2) - door_x_delta;
            blocks[2][y][x] = VoxelBlock::EMPTY;
        }
    }

    // ceiling
    for y_delta in 0..=7 {
        let y = floor_height + 1 + wall_height + y_delta;
        for x in (1 + y_delta)..CHUNK_DIM - (1 + y_delta) {
            blocks[1 + y_delta][y][x] = solid_block;
            blocks[CHUNK_DIM - 1 - (1 + y_delta)][y][x] = solid_block;
            blocks[x][y][1 + y_delta] = solid_block;
            blocks[x][y][CHUNK_DIM - 1 - (1 + y_delta)] = solid_block;
        }
    }

    // chimney
    for y in 0..CHUNK_DIM - 1 {
        blocks[2][y][2] = solid_block;
        blocks[2][y][3] = solid_block;
        blocks[3][y][2] = solid_block;
        blocks[3][y][3] = solid_block;
    }
    for x in 1..5 {
        blocks[1][CHUNK_DIM - 1][x] = solid_block;
        blocks[4][CHUNK_DIM - 1][x] = solid_block;
        blocks[x][CHUNK_DIM - 1][1] = solid_block;
        blocks[x][CHUNK_DIM - 1][4] = solid_block;
    }

    // junk
    for y_delta in 0..3 {
        let y = floor_height + 2 + (y_delta * 2);
        for x_delta in 0..2 {
            let x = 4 + (x_delta * 2);
            for z_delta in 0..2 {
                let z = 4 + (z_delta * 2);
                blocks[z][y][x] = solid_block;
                blocks[z][y][CHUNK_DIM - 1 - x] = solid_block;
                blocks[CHUNK_DIM - 1 - z][y][x] = solid_block;
                blocks[CHUNK_DIM - 1 - z][y][CHUNK_DIM - 1 - x] = solid_block;
            }
        }
    }

    let voxel_count = blocks
        .iter()
        .flatten()
        .flatten()
        .filter(|b| b.is_transparent())
        .count() as u32;

    VoxelChunk::new(position, blocks, voxel_count)
}
