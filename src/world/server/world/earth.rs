use crate::world::generation::{WorldConfig, WorldGenHandle, WorldGenRequest};
use crate::world::server::world::r#trait::World;
use crate::world::types::Chunk;
use glam::IVec3;
use rustc_hash::{FxHashMap, FxHashSet};

pub struct Earth {
    config: WorldConfig,
    chunks: FxHashMap<IVec3, Chunk>,
    generation_handle: WorldGenHandle,
    generation_request_batch: FxHashSet<IVec3>,
}

impl Earth {
    pub fn new(config: WorldConfig, chunks_size_hint: usize) -> Self {
        let mut chunks = FxHashMap::default();
        chunks.reserve(chunks_size_hint);
        Self {
            config,
            chunks,
            generation_handle: WorldGenHandle::new(config),
            generation_request_batch: FxHashSet::default(),
        }
    }
}

impl World for Earth {
    fn tick(&mut self) {
        if let Ok(new_chunks) = self.generation_handle.try_recv() {
            self.chunks.extend(new_chunks.into_iter());
        }
    }

    fn request_chunks(&mut self, positions: &[IVec3]) -> Vec<&Chunk> {
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
            .send(WorldGenRequest::new(positions))
            .expect("Failed to send generation request");
    }

    fn start_simulation(&mut self) {
        self.generation_handle.start_thread();
    }

    fn stop_simulation(&mut self) {
        self.generation_handle.stop_thread()
    }
}
