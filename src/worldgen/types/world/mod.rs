mod chunk_generation;

use crate::render::types::Mesh;
use crate::worldgen::types::Chunk;
use glam::IVec3;
use parking_lot::RwLock;
use std::sync::Arc;
use wgpu::naga::{FastHashMap, FastHashSet};

#[derive(Default)]
pub struct World {
    pub(crate) noise: Arc<noise::OpenSimplex>, // hard code noise func here for now
    pub seed: u32,
    pub chunks: FastHashMap<IVec3, Chunk>,
    pub generating_chunks: Arc<RwLock<FastHashSet<IVec3>>>,
    pub generating_meshes: Arc<RwLock<FastHashSet<IVec3>>>,
}

impl World {
    pub fn new(seed: u32) -> Self {
        Self {
            noise: Arc::new(noise::OpenSimplex::new(seed)),
            seed,
            chunks: FastHashMap::default(),
            generating_chunks: Arc::new(RwLock::new(FastHashSet::default())),
            generating_meshes: Arc::new(RwLock::new(FastHashSet::default())),
        }
    }

    // pub fn generate(&mut self, texture_atlas: &TextureAtlas) {
    //     const world_size: f32 = 32f32; // fixme temp const
    //
    //     for x in -(world_size / 2f32) as i32..(world_size / 2f32) as i32 {
    //         for y in 0..1 {
    //             for z in -(world_size / 2f32) as i32..(world_size / 2f32) as i32 {
    //                 let chunk_position = Vec3::new(x as f32, y as f32, z as f32);
    //                 let blocks = World::generate_chunk_blocks(&ns, chunk_position);
    //                 let mesh = Chunk::generate_mesh(&blocks, texture_atlas);
    //                 let chunk = Chunk { blocks, mesh };
    //                 self.chunks.insert(IVec3::new(x, y, z), chunk);
    //             }
    //         }
    //     }
    // }

    pub fn get_meshes_for_positions(&self, positions: Vec<&IVec3>) -> Vec<Option<Mesh>> {
        let mut meshes = Vec::with_capacity(positions.len());
        for pos in positions {
            let c = self.chunks.get(pos).unwrap();
            meshes.push(c.mesh.clone());
        }
        meshes
    }
}
