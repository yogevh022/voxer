use crate::compute;
use crate::compute::array::Array3D;
use crate::compute::geo::Plane;
use crate::world::session::PlayerSession;
use crate::world::types::Chunk;
use glam::IVec3;
use rustc_hash::{FxHashMap, FxHashSet};
use std::time::Instant;

pub struct ClientWorldSession {
    pub(crate) player: PlayerSession,
    pub(crate) view_frustum: [Plane; 6],
    pub(crate) chunks: FxHashMap<IVec3, Chunk>,
    pub(crate) missing_chunks: Option<Vec<IVec3>>,
    last_request_chunk_positions: FxHashMap<IVec3, Instant>,
    unprocessed_chunk_positions: Vec<IVec3>,
}

impl ClientWorldSession {
    pub fn new(player: PlayerSession) -> Self {
        Self {
            player,
            view_frustum: [Plane::default(); 6],
            chunks: FxHashMap::default(),
            missing_chunks: None,
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
            .map(|instant| instant.elapsed().as_millis() > 400)
            .unwrap_or(true)
        {
            self.last_request_chunk_positions
                .insert(position, Instant::now());
            return true;
        }
        false
    }

    pub fn tick(&mut self) {
        let positions = std::mem::take(&mut self.unprocessed_chunk_positions);
        let mut updated = FxHashSet::default();
        for position in positions
            .into_iter()
            .map(|p| extended_with_preceding_positions(p))
            .flatten()
        {
            if updated.contains(&position) {
                continue;
            }
            updated.insert(position);
            self.update_chunk_data(position);
        }
    }

    fn update_chunk_data(&mut self, position: IVec3) {
        let adjacent_blocks = Array3D(compute::chunk::get_adjacent_blocks(position, &self.chunks));
        self.chunks.get_mut(&position).map(|chunk| {
            chunk.face_count = Some(compute::chunk::face_count(&chunk.blocks, &adjacent_blocks));
            chunk.adjacent_blocks = adjacent_blocks;
        });
    }
}

fn extended_with_preceding_positions(origin: IVec3) -> [IVec3; 4] {
    [
        origin,
        IVec3::new(origin.x - 1, origin.y, origin.z),
        IVec3::new(origin.x, origin.y - 1, origin.z),
        IVec3::new(origin.x, origin.y, origin.z - 1),
    ]
}
