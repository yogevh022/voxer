use crate::compute::array::Array3D;
use crate::world::types::chunk_manager::ChunkManager;
use crate::world::types::{Block, CHUNK_DIM, Chunk, ChunkBlocks};
use glam::{IVec3, Vec3};
use indexmap::{IndexMap, IndexSet};
use noise::NoiseFn;
use std::collections::{HashMap, HashSet};

const NOISE_SCALE: f64 = 0.05;

#[derive(Default, Debug)]
pub struct World {
    pub seed: u32,
    pub chunk_manager: ChunkManager,
}

impl World {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            chunk_manager: ChunkManager::with_capacity(1000), //fixme
        }
    }

    pub fn generate_chunk(noise: impl NoiseFn<f64, 3>, chunk_position: IVec3) -> Chunk {
        let blocks = World::generate_chunk_blocks(noise, chunk_position);
        Chunk {
            blocks,
            mesh: None,
            last_visited: None,
        }
    }

    fn generate_chunk_blocks(noise: impl NoiseFn<f64, 3>, chunk_position: IVec3) -> ChunkBlocks {
        let blocks: ChunkBlocks = Array3D(std::array::from_fn(|x| {
            std::array::from_fn(|y| {
                std::array::from_fn(|z| {
                    if noise.get([
                        (chunk_position.x * CHUNK_DIM as i32 + x as i32) as f64 * NOISE_SCALE,
                        (chunk_position.y * CHUNK_DIM as i32 + y as i32) as f64 * NOISE_SCALE,
                        (chunk_position.z * CHUNK_DIM as i32 + z as i32) as f64 * NOISE_SCALE,
                    ]) > 0.1
                    {
                        // fixme this is horrible
                        Block(1u16 << 15)
                    } else {
                        Block(1u16)
                    }
                })
            })
        }));
        blocks
    }

    pub(crate) fn world_to_chunk_pos(vec: &Vec3) -> IVec3 {
        let chunk_pos_float = vec / CHUNK_DIM as f32;
        IVec3::new(
            chunk_pos_float.x as i32,
            chunk_pos_float.y as i32,
            chunk_pos_float.z as i32,
        )
    }

    pub(crate) fn chunk_to_world_pos(chunk_pos: &IVec3) -> Vec3 {
        let world_pos_round = chunk_pos * CHUNK_DIM as i32;
        Vec3::new(
            world_pos_round.x as f32,
            world_pos_round.y as f32,
            world_pos_round.z as f32,
        )
    }
}
