mod meshing;
pub mod types;

use crate::worldgen::types::{BlockKind, CHUNK_SIZE, Chunk};
use glam::Vec3;
use noise;
use noise::NoiseFn;
use std::sync::Arc;

const NOISE_SCALE: f64 = 0.05;

pub fn generate_chunk(ns: &Arc<impl NoiseFn<f64, 3>>, chunk_position: Vec3) -> Chunk {
    let blocks: [[[BlockKind; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE] = std::array::from_fn(|x| {
        std::array::from_fn(|y| {
            std::array::from_fn(|z| {
                if ns.get([
                    (chunk_position.x as f64 * CHUNK_SIZE as f64 + x as f64) * NOISE_SCALE,
                    (chunk_position.y as f64 * CHUNK_SIZE as f64 + y as f64) * NOISE_SCALE,
                    (chunk_position.z as f64 * CHUNK_SIZE as f64 + z as f64) * NOISE_SCALE,
                ]) > 0.1
                {
                    BlockKind::Stone
                } else {
                    BlockKind::Air
                }
            })
        })
    });
    Chunk { blocks }
}
