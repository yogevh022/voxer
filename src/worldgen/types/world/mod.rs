use crate::render::types::Mesh;
use crate::texture::TextureAtlas;
use crate::worldgen::types::Chunk;
use crate::worldgen;
use glam::{IVec3, Vec3};
use std::sync::Arc;
use wgpu::naga::FastHashMap;

#[derive(Default)]
pub struct World {
    pub seed: u32,
    pub chunks: FastHashMap<IVec3, Chunk>,
    pub loaded_chunks: Vec<IVec3>,
}

impl World {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            chunks: FastHashMap::default(),
            loaded_chunks: Vec::new(),
        }
    }

    pub fn generate(&mut self, texture_atlas: &TextureAtlas) {
        const world_size: f32 = 32f32; // fixme temp const

        let ns = Arc::new(noise::OpenSimplex::new(self.seed));
        for x in -(world_size / 2f32) as i32..(world_size / 2f32) as i32 {
            for y in 0..1 {
                for z in -(world_size / 2f32) as i32..(world_size / 2f32) as i32 {
                    let chunk_position = Vec3::new(x as f32, y as f32, z as f32);
                    let blocks = worldgen::generate_chunk_blocks(&ns, chunk_position);
                    let mesh = Chunk::generate_mesh(&blocks, texture_atlas);
                    let chunk = Chunk { blocks, mesh };
                    self.chunks.insert(IVec3::new(x, y, z), chunk);
                }
            }
        }
    }

    pub fn get_loaded_chunks(&self) -> Vec<&Chunk> {
        let mut chunks = Vec::new();
        for cpos in self.loaded_chunks.iter() {
            let c = self.chunks.get(cpos).unwrap();
            chunks.push(c);
        }
        chunks
    }

    pub fn get_meshes_for_positions(&self, positions: Vec<&IVec3>) -> Vec<Mesh> {
        let mut meshes = Vec::with_capacity(positions.len());
        for pos in positions {
            let c = self.chunks.get(pos).unwrap();
            meshes.push(c.mesh.clone());
        }
        meshes
    }
}
