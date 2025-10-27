use crate::world::server::world::{World, WorldConfig};
use crate::world::server::world::earth_gen::EarthGen;
use crate::world::server::world::generation::{
    WorldGenHandle, WorldGenRequest, WorldGenResponse,
};
use glam::IVec3;
use rustc_hash::{FxHashMap, FxHashSet};
use crate::world::server::world::chunk::VoxelChunk;

pub struct Earth {
    config: WorldConfig,
    chunks: FxHashMap<IVec3, VoxelChunk>,
    generation_handle: WorldGenHandle<EarthGen>,
    generation_request_batch: FxHashSet<IVec3>,
}

impl Earth {
    pub fn new(config: WorldConfig, chunks_size_hint: usize) -> Self {
        let mut chunks = FxHashMap::default();
        chunks.reserve(chunks_size_hint);
        let earth_gen = EarthGen::new(config.clone());
        Self {
            config,
            chunks,
            generation_handle: WorldGenHandle::new(earth_gen),
            generation_request_batch: FxHashSet::default(),
        }
    }
}

impl World for Earth {
    fn tick(&mut self) {
        if let Ok(gen_response) = self.generation_handle.try_recv() {
            match gen_response {
                WorldGenResponse::Chunks(chunks) => {
                    self.chunks
                        .extend(chunks.into_iter().map(|chunk| (chunk.position, chunk)));
                }
                _ => {}
            }
        }
    }

    fn request_chunks(&mut self, positions: &[IVec3]) -> Vec<&VoxelChunk> {
        let mut chunks = Vec::with_capacity(positions.len());
        for position in positions {
            match self.chunks.get(position) {
                Some(chunk) => chunks.push(chunk),
                None => {
                    self.generation_request_batch.insert(*position);
                }
            };
        }
        chunks
    }

    fn request_chunk_generation(&mut self) {
        let positions: Vec<IVec3> = self
            .generation_request_batch
            .iter()
            .filter_map(|p| (!self.generation_handle.is_pending(p)).then(|| *p))
            .collect();
        self.generation_request_batch.clear();
        if positions.is_empty() {
            return;
        }
        self.generation_handle
            .send(WorldGenRequest::Chunks(positions))
            .expect("Failed to send generation request");
    }

    fn start_simulation(&mut self) {
        self.generation_handle.start_thread();
    }

    fn stop_simulation(&mut self) {
        self.generation_handle.stop_thread()
    }
}
