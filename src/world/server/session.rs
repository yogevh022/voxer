use crate::compute;
use crate::world::server::world::World;
use crate::world::session::PlayerSession;
use glam::IVec3;
use rustc_hash::FxHashMap;
use std::net::SocketAddr;
use crate::world::server::world::chunk::VoxelChunk;

pub(crate) struct ServerPlayerSession {
    pub player: PlayerSession,
    pub addr: SocketAddr,
}

pub(crate) struct ServerWorldSession {
    worlds: Vec<Box<dyn World>>,
    pub(crate) players: FxHashMap<usize, ServerPlayerSession>,
    addr_to_player: FxHashMap<SocketAddr, usize>,
}

impl ServerWorldSession {
    pub(crate) fn new(worlds: Vec<Box<dyn World>>) -> Self {
        Self {
            worlds,
            players: FxHashMap::default(),
            addr_to_player: FxHashMap::default(),
        }
    }

    pub(crate) fn tick(&mut self) {
        for world in self.worlds.iter_mut() {
            world.tick();
            world.request_chunk_generation();
        }
    }

    pub(crate) fn add_player(&mut self, player_session: ServerPlayerSession) {
        self.addr_to_player
            .insert(player_session.addr, player_session.player.id);
        self.players
            .insert(player_session.player.id, player_session);
    }

    pub(crate) fn remove_player(&mut self, player_id: usize) {
        // todo remove from addr_to_player here?
        self.players.remove(&player_id);
    }

    pub(crate) fn player_by_addr(&self, addr: SocketAddr) -> Option<usize> {
        self.addr_to_player.get(&addr).copied()
    }

    pub(crate) fn request_chunks_from_world(
        &mut self,
        world_index: usize,
        chunk_positions: &[IVec3],
    ) -> Vec<&VoxelChunk> {
        self.worlds[world_index].request_chunks(chunk_positions)
    }

    pub(crate) fn start(&mut self) {
        self.worlds.first_mut().unwrap().start_simulation();
    }
}
