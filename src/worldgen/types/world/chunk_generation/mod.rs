mod chunk_task;

use crate::worldgen::types::{BlockKind, CHUNK_SIZE, Chunk, ChunkBlocks, World};
use chunk_task::{ChunkTaskKind, ChunkTasks, task_kind_for};
use glam::IVec3;
use noise::NoiseFn;
use parking_lot::RwLock;
use std::sync::Arc;
use wgpu::naga::FastHashSet;

const NOISE_SCALE: f64 = 0.05;

impl World {
    pub(crate) fn chunks_tasks<'a>(&self, chunk_positions: &'a [IVec3]) -> ChunkTasks<'a> {
        let mut renderer_load = Vec::new();
        let mut generate_chunk = Vec::new();
        let mut generate_mesh = Vec::new();

        for chunk_pos in chunk_positions {
            let task_kind = task_kind_for(self.chunks.get(chunk_pos));
            match task_kind {
                Some(ChunkTaskKind::RendererLoad) => renderer_load.push(chunk_pos),
                Some(ChunkTaskKind::GenerateMesh) => generate_mesh.push(chunk_pos),
                Some(ChunkTaskKind::GenerateChunk) => generate_chunk.push(chunk_pos),
                None => continue,
            }
        }

        ChunkTasks {
            renderer_load,
            generate_chunk,
            generate_mesh,
        }
    }

    pub fn generate_chunk_blocks(
        ns: &Arc<impl NoiseFn<f64, 3>>,
        chunk_position: IVec3,
    ) -> ChunkBlocks {
        let blocks: ChunkBlocks = std::array::from_fn(|x| {
            std::array::from_fn(|y| {
                std::array::from_fn(|z| {
                    if ns.get([
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

    pub fn generate_chunk(noise: &Arc<impl NoiseFn<f64, 3>>, chunk_position: IVec3) -> Chunk {
        let blocks = World::generate_chunk_blocks(noise, chunk_position);
        Chunk {
            blocks,
            mesh: None,
            last_visited: None,
        }
    }
}
