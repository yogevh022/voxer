use crate::worldgen::types::{BlockKind, CHUNK_SIZE, Chunk, ChunkBlocks};
use glam::IVec3;
use noise::NoiseFn;
use std::collections::HashMap;
use wgpu::naga::FastHashSet;

const NOISE_SCALE: f64 = 0.05;

#[derive(Debug, Default, Clone)]
pub struct WorldGenerationState {
    pub chunks: FastHashSet<IVec3>,
    pub meshes: FastHashSet<IVec3>,
}

pub struct ChunksStatus {
    pub to_render: Vec<IVec3>,
    pub not_found: Vec<IVec3>,
    pub meshless: Vec<IVec3>,
}

#[derive(Default, Debug)]
pub struct World {
    pub seed: u32,
    pub chunks: HashMap<IVec3, Option<Chunk>>,
    pub active_generation: WorldGenerationState,
}

impl World {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            chunks: HashMap::default(),
            active_generation: WorldGenerationState::default(),
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
        let blocks: ChunkBlocks = std::array::from_fn(|x| {
            std::array::from_fn(|y| {
                std::array::from_fn(|z| {
                    if noise.get([
                        (chunk_position.x * CHUNK_SIZE as i32 + x as i32) as f64 * NOISE_SCALE,
                        (chunk_position.y * CHUNK_SIZE as i32 + y as i32) as f64 * NOISE_SCALE,
                        (chunk_position.z * CHUNK_SIZE as i32 + z as i32) as f64 * NOISE_SCALE,
                    ]) > 0.1
                    {
                        BlockKind::Stone
                    } else {
                        BlockKind::Air
                    }
                })
            })
        });
        blocks
    }

    pub fn chunks_status(&self, chunk_positions: Vec<IVec3>) -> ChunksStatus {
        let mut to_render = Vec::new();
        let mut not_found = Vec::new();
        let mut meshless = Vec::new();

        for pos in chunk_positions {
            match self.chunks.get(&pos) {
                Some(Some(chunk)) => match &chunk.mesh {
                    Some(mesh) => {
                        if !mesh.indices.is_empty() {
                            to_render.push(pos)
                        }
                    }
                    None => {
                        if !self.active_generation.meshes.contains(&pos) {
                            meshless.push(pos)
                        }
                    }
                },
                Some(None) => {} // this chunk's mesh is being generated
                None => {
                    if !self.active_generation.chunks.contains(&pos) {
                        not_found.push(pos)
                    }
                }
            }
        }

        ChunksStatus {
            to_render,
            not_found,
            meshless,
        }
    }
}
