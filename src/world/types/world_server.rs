use crate::compute::geo;
use crate::vtypes::Scene;
use crate::world::generation::{WorldGenConfig, WorldGenHandle};
use crate::world::types::{Block, CHUNK_DIM, Chunk, ChunkBlocks};
use glam::{IVec3, Vec3};
use std::collections::{HashMap, HashSet};

pub struct WorldServerConfig {
    pub seed: i32,
    pub simulation_distance: usize,
}

pub struct WorldServer {
    generation_handle: WorldGenHandle,
    chunks: HashMap<IVec3, Chunk>,
    simulated_chunks: HashSet<IVec3>,
    scene: Scene,
    players: HashMap<usize, Vec3>,
    config: WorldServerConfig,
}

impl WorldServer {
    pub fn new(config: WorldServerConfig) -> Self {
        let world_gen_config = WorldGenConfig {
            seed: config.seed,
            noise_scale: 0.05,
        };
        Self {
            generation_handle: WorldGenHandle::new(world_gen_config),
            chunks: Default::default(),
            simulated_chunks: Default::default(),
            scene: Default::default(),
            players: HashMap::new(),
            config,
        }
    }

    pub(crate) fn start_generation_thread(&mut self) {
        self.generation_handle.start_thread();
    }

    pub(crate) fn update(&mut self) {
        let mut active_chunk_positions = Vec::new();
        for (_, player_pos) in self.players.iter() {
            geo::Sphere::discrete_points(
                geo::world_to_chunk_pos(*player_pos),
                self.config.simulation_distance as isize,
                |point| {
                    active_chunk_positions.push(point);
                },
            );
        }
        // active_chunk_positions.push(IVec3::new(0, 0, 0));
        // active_chunk_positions.push(IVec3::new(1, 0, 0));
        // active_chunk_positions.push(IVec3::new(2, 0, 0));
        // active_chunk_positions.push(IVec3::new(3, 0, 0));
        self.try_receive_generation();
        let (generated, ungenerated): (HashSet<_>, HashSet<_>) =
            self.partition_chunks_by_existence(active_chunk_positions);
        self.request_generation(ungenerated);
        self.simulated_chunks = generated;
    }

    pub fn set_player(&mut self, player_id: usize, player_pos: Vec3) {
        self.players.insert(player_id, player_pos);
    }

    pub fn get_chunks(&self, positions: Vec<IVec3>) -> Vec<Chunk> {
        // cloning here because the server will have to send clones to clients anyway
        // only returns chunks that are generated
        positions
            .into_iter()
            .filter_map(|c_pos| self.chunks.get(&c_pos).cloned())
            .collect()
    }

    fn partition_chunks_by_existence(
        &self,
        chunk_positions: Vec<IVec3>,
    ) -> (HashSet<IVec3>, HashSet<IVec3>) {
        chunk_positions.into_iter().partition(|c_pos| {
            self.chunks.contains_key(c_pos) || self.generation_handle.is_pending(c_pos)
        })
    }

    fn try_receive_generation(&mut self) {
        if let Ok(new_chunks) = self.generation_handle.try_recv() {
            self.chunks.extend(new_chunks.into_iter());
        }
    }

    fn request_generation(&mut self, chunk_positions: HashSet<IVec3>) {
        self.generation_handle
            .send(chunk_positions.into_iter().collect::<Vec<_>>())
            .expect("Failed to send generation request");
    }
}
