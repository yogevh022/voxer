use crate::app::app_renderer;
use crate::app::app_renderer::AppRenderer;
use crate::compute;
use crate::compute::chunk::TRANSPARENT_LAYER_BLOCKS;
use crate::compute::geo;
use crate::world::types::{Block, CHUNK_DIM, Chunk};
use glam::{IVec3, Vec3};
use rustc_hash::{FxHashMap, FxHashSet};
use std::array;
use std::sync::Arc;
use winit::window::Window;
use crate::compute::array::Array3D;

pub struct WorldClientConfig {
    pub render_distance: usize,
}

pub struct WorldClient<'window> {
    pub config: WorldClientConfig,
    pub renderer: AppRenderer<'window, 2, 1>,
    pub chunks: FxHashMap<IVec3, Chunk>,
    player_position: Vec3,
}

impl<'window> WorldClient<'window> {
    pub fn new(window: Arc<Window>, config: WorldClientConfig) -> Self {
        Self {
            renderer: app_renderer::make_app_renderer(window),
            chunks: FxHashMap::default(),
            config,
            player_position: Vec3::ZERO,
        }
    }

    fn get_chunk_render(&self, pos: IVec3) -> ChunkRelevantBlocks {
        let px = IVec3::new(pos.x + 1, pos.y, pos.z);
        let py = IVec3::new(pos.x, pos.y + 1, pos.z);
        let pz = IVec3::new(pos.x, pos.y, pos.z + 1);
        let chunk = self.chunks.get(&pos).unwrap().clone();

        let adjacent_blocks = Array3D([
            self.chunks.get(&px).map_or(TRANSPARENT_LAYER_BLOCKS, |c| {
                compute::chunk::get_mx_layer(&c.blocks)
            }),
            self.chunks.get(&py).map_or(TRANSPARENT_LAYER_BLOCKS, |c| {
                compute::chunk::get_my_layer(&c.blocks)
            }),
            self.chunks.get(&pz).map_or(TRANSPARENT_LAYER_BLOCKS, |c| {
                compute::chunk::get_mz_layer(&c.blocks)
            }),
        ]);
        ChunkRelevantBlocks {
            chunk,
            adjacent_blocks,
        }
    }

    pub fn add_chunks(&mut self, chunks: Vec<Chunk>) {
        for chunk in chunks {
            self.chunks.insert(chunk.position, chunk);
        }
    }

    pub fn remove_chunks(&mut self, positions: Vec<IVec3>) {
        for pos in positions {
            self.chunks.remove(&pos);
        }
    }
    
    pub fn chunks_to_mesh(&self, frustum_planes: &[geo::Plane; 6]) -> Vec<IVec3>
    {
        let mut added_chunks = FxHashSet::default();
        let mut positions = Vec::new();
        
        geo::Sphere::discrete_points(
            geo::world_to_chunk_pos(self.player_position),
            self.config.render_distance as isize,
            |chunk_position| {
                // todo optimize this
                let chunk_world_position = geo::chunk_to_world_pos(chunk_position);
                if geo::Frustum::is_aabb_within_frustum(
                    chunk_world_position,
                    chunk_world_position + CHUNK_DIM as f32,
                    &frustum_planes,
                ) && !self.renderer.is_chunk_rendered(chunk_position) {
                    added_chunks.insert(chunk_position);
                    positions.push(chunk_position);
                    let mx_chunk_position = IVec3::new(chunk_position.x - 1, chunk_position.y, chunk_position.z);
                    let my_chunk_position = IVec3::new(chunk_position.x, chunk_position.y - 1, chunk_position.z);
                    let mz_chunk_position = IVec3::new(chunk_position.x, chunk_position.y, chunk_position.z - 1);
                    if added_chunks.insert(mx_chunk_position) {
                        positions.push(mx_chunk_position);
                    }
                    if added_chunks.insert(my_chunk_position) {
                        positions.push(my_chunk_position);
                    }
                    if added_chunks.insert(mz_chunk_position) {
                        positions.push(mz_chunk_position);
                    }
                };
            },
        );
        positions
    }

    pub fn chunk_rel_blocks(&self, positions: Vec<IVec3>) -> Vec<ChunkRelevantBlocks> {
        let mut renders = Vec::new();
        for pos in positions {
            if self.chunks.contains_key(&pos) {
                renders.push(self.get_chunk_render(pos));
            }
        }
        renders
    }

    pub fn set_player_position(&mut self, position: Vec3) {
        self.player_position = position;
    }
}
pub type ChunkAdjacentBlocks = Array3D<Block, 3, CHUNK_DIM, CHUNK_DIM>;

pub struct ChunkRelevantBlocks {
    pub chunk: Chunk,
    pub adjacent_blocks: ChunkAdjacentBlocks,
}
