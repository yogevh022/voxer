use crate::compute::geo;
use crate::world::generation::{WorldConfig, WorldGenHandle, WorldGenRequest};
use crate::world::server::world::r#trait::World;
use crate::world::types::Chunk;
use glam::IVec3;
use rustc_hash::{FxHashMap, FxHashSet};

pub struct Earth {
    chunks: FxHashMap<IVec3, Chunk>,
    simulated_chunks: FxHashSet<IVec3>,
    generation_handle: WorldGenHandle,
    config: WorldConfig,
}

impl Earth {
    pub fn new(config: WorldConfig) -> Self {
        Self {
            chunks: FxHashMap::default(),
            simulated_chunks: FxHashSet::default(),
            generation_handle: WorldGenHandle::new(config),
            config,
        }
    }

    fn request_generation(&mut self, chunk_positions: Vec<IVec3>) {
        let request = WorldGenRequest::new(chunk_positions);
        self.generation_handle
            .send(request)
            .expect("Failed to send generation request");
    }

    fn chunk_registered(&self, position: IVec3) -> bool {
        self.chunks.contains_key(&position) || self.generation_handle.is_pending(&position)
    }
}

impl World for Earth {
    fn tick(&mut self) {
        if let Ok(new_chunks) = self.generation_handle.try_recv() {
            self.chunks.extend(new_chunks.into_iter());
        }
    }

    fn chunks_at(&self, positions: &[IVec3]) -> Vec<&Chunk> {
        // only returns chunks that are generated
        positions
            .into_iter()
            .filter_map(|c_pos| self.chunks.get(&c_pos))
            .collect()
    }

    fn update_simulated_chunks(&mut self, origins: &[IVec3]) {
        self.simulated_chunks.clear();
        let mut generation_requests = Vec::new();
        for origin in origins {
            geo::Sphere::discrete_points(
                *origin,
                self.config.simulation_distance as isize,
                |point| {
                    if self.chunk_registered(point) {
                        self.simulated_chunks.insert(point);
                    } else {
                        generation_requests.push(point);
                    }
                },
            );
        }
        self.request_generation(generation_requests);
    }

    fn start_simulation(&mut self) {
        self.generation_handle.start_thread();
    }

    fn stop_simulation(&mut self) {
        self.generation_handle.stop_thread()
    }
}
