use std::net::SocketAddr;
use glam::IVec3;
use rustc_hash::FxHashMap;
use crate::compute;
use crate::world::server::world::World;
use crate::world::session::PlayerSession;
use crate::world::types::Chunk;

pub(crate) struct ServerPlayerSession {
    pub player: PlayerSession,
    pub addr: SocketAddr,
}

pub(crate) struct ServerWorldSession {
    worlds: Vec<Box<dyn World>>,
    players: FxHashMap<usize, ServerPlayerSession>,
}

impl ServerWorldSession {
    pub(crate) fn new(worlds: Vec<Box<dyn World>>) -> Self {
        Self {
            worlds,
            players: FxHashMap::default(),
        }
    }

    pub(crate) fn tick(&mut self) {
        let player_locations = self.get_player_locations();
        for (world_index, origins) in player_locations {
            self.worlds[world_index].update_simulated_chunks(&origins);
        }
        for world in self.worlds.iter_mut() {
            world.tick();
        }
    }

    pub(crate) fn add_player(&mut self, player_session: ServerPlayerSession) {
        self.players.insert(player_session.player.id, player_session);
    }

    pub(crate) fn remove_player(&mut self, player_id: usize) {
        self.players.remove(&player_id);
    }

    pub(crate) fn get_chunks(&self, world_index: usize, chunk_positions: &[IVec3]) -> Vec<&Chunk> {
        self.worlds[world_index].chunks_at(chunk_positions)
    }

    pub(crate) fn start(&mut self) {
        self.worlds.first_mut().unwrap().start_simulation();
    }

    fn get_player_locations(&self) -> FxHashMap<usize, Vec<IVec3>> {
        let mut locations = FxHashMap::<usize, Vec<IVec3>>::default();
        for player_session in self.players.values() {
            locations
                .entry(player_session.player.location.world)
                .or_default()
                .push(compute::geo::world_to_chunk_pos(player_session.player.location.position));
        }
        locations
    }
}