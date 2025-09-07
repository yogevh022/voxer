use crate::compute;
use crate::compute::array::Array3D;
use crate::compute::geo::Plane;
use crate::world::session::PlayerSession;
use crate::world::types::Chunk;
use glam::IVec3;
use rustc_hash::FxHashMap;
use std::time::Instant;

pub struct ClientWorldSession {
    pub(crate) player: PlayerSession,
    pub(crate) view_frustum: [Plane; 6],
    pub(crate) chunks: FxHashMap<IVec3, Chunk>,
    last_request_chunk_positions: FxHashMap<IVec3, Instant>,
    unprocessed_chunk_positions: Vec<IVec3>,
}

impl ClientWorldSession {
    pub fn new(player: PlayerSession) -> Self {
        Self {
            player,
            view_frustum: [Plane::default(); 6],
            chunks: FxHashMap::default(),
            last_request_chunk_positions: FxHashMap::default(),
            unprocessed_chunk_positions: Vec::new(),
        }
    }

    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.unprocessed_chunk_positions.push(chunk.position);
        self.chunks.insert(chunk.position, chunk);
    }

    pub fn try_request_permission(&mut self, position: IVec3) -> bool {
        if self
            .last_request_chunk_positions
            .get(&position)
            .map(|instant| instant.elapsed().as_millis() > 1000)
            .unwrap_or(true)
        {
            self.last_request_chunk_positions.insert(position, Instant::now());
            return true;
        }
        false
    }

    pub fn tick(&mut self) {
        for position in std::mem::take(&mut self.unprocessed_chunk_positions) {
            self.update_chunk_data(position);
            self.update_adjacent_chunks_data(position);
        }
    }

    fn update_chunk_data(&mut self, position: IVec3) {
        let adjacent_blocks = Array3D(compute::chunk::get_adjacent_blocks(position, &self.chunks));
        self.chunks.get_mut(&position).map(|chunk| {
            chunk.face_count = Some(compute::chunk::face_count(&chunk.blocks, &adjacent_blocks));
            chunk.adjacent_blocks = adjacent_blocks;
        });
    }

    fn update_adjacent_chunks_data(&mut self, origin_position: IVec3) {
        // fixme fix performance redundancy
        for chunk_position in [
            IVec3::new(origin_position.x - 1, origin_position.y, origin_position.z),
            IVec3::new(origin_position.x, origin_position.y - 1, origin_position.z),
            IVec3::new(origin_position.x, origin_position.y, origin_position.z - 1),
        ] {
            self.update_chunk_data(chunk_position);
        }
    }
}
