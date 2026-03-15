use crate::compute::geo::{Sphere, SpherePointsRange, ivec3_with_adjacent_positions};
use crate::compute::utils::fxmap_with_capacity;
use crate::vtypes::Camera;
use crate::world::ClientWorldConfig;
use crate::world::server::chunk::VoxelChunk;
use glam::IVec3;
use rustc_hash::{FxHashMap, FxHashSet};

pub struct ClientWorldSession {
    pub chunks: FxHashMap<IVec3, VoxelChunk>,
    pub camera: Camera,
    config: ClientWorldConfig,
    chunk_drop_dist: i32,
    chunk_gc_batch: Vec<IVec3>,
    chunk_interest_positions: SpherePointsRange,
    chunk_meshing_batch: FxHashSet<IVec3>,
}

impl ClientWorldSession {
    pub fn new(config: ClientWorldConfig, start_position: IVec3) -> Self {
        Self {
            chunks: fxmap_with_capacity((config.render_distance * 2).pow(3)),
            camera: Camera::default(),
            config,
            chunk_drop_dist: (config.render_distance as i32).pow(2) + 1,
            chunk_gc_batch: Vec::new(),
            chunk_interest_positions: Sphere::discrete_points(
                start_position,
                config.render_distance as u32 - 1,
            ), // fixme
            chunk_meshing_batch: FxHashSet::default(),
        }
    }

    pub fn add_new_chunk(&mut self, chunk: VoxelChunk) {
        self.chunk_meshing_batch
            .extend(ivec3_with_adjacent_positions(chunk.position));
        self.chunks.insert(chunk.position, chunk);
    }

    pub fn chunk_exists(&self, chunk_position: &IVec3) -> bool {
        self.chunks.contains_key(chunk_position)
    }

    pub fn chunk_gc_pass(&mut self, camera_ch_position: IVec3) {
        if self.chunk_gc_batch.is_empty() {
            self.chunk_gc_batch.extend(self.chunks.keys());
        }
        const CHUNKS_PER_PASS: usize = 16; // fixme add to centralized config
        let position_range = self.chunk_gc_batch.len().saturating_sub(CHUNKS_PER_PASS)..;
        for position in self.chunk_gc_batch.drain(position_range) {
            if camera_ch_position.distance_squared(position) > self.chunk_drop_dist {
                self.chunks.remove(&position);
            }
        }
    }

    pub fn missing_interest_chunks<F: FnMut(IVec3)>(
        &mut self,
        chunk_origin: IVec3,
        count: usize,
        mut f: F,
    ) {
        if self.chunk_interest_positions.is_empty() {
            self.chunk_interest_positions =
                Sphere::discrete_points(chunk_origin, self.config.render_distance as u32 - 1);
        }
        for pos in (&mut self.chunk_interest_positions).take(count) {
            if !self.chunks.contains_key(&pos) {
                f(pos);
            }
        }
    }

    pub fn chunk_meshing_batch(&mut self) -> impl Iterator<Item = &VoxelChunk> {
        self.chunk_meshing_batch
            .drain()
            .filter_map(|p| self.chunks.get(&p))
    }

    pub fn tick(&mut self, camera_origin: IVec3) {
        self.chunk_gc_pass(camera_origin);
    }
}
