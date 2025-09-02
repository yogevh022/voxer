mod config;
mod network;
mod player;
mod world;
pub use config::WorldServerConfig;

use crate::compute;
use crate::vtypes::VoxerUdpSocket;
use crate::world::generation::WorldConfig;
use crate::world::server::player::Player;
use crate::world::server::world::{Earth, World};
use glam::IVec3;
use rustc_hash::FxHashMap;

pub struct WorldServer {
    worlds: Vec<Box<dyn World>>,
    players: FxHashMap<usize, Player>,
    network: VoxerUdpSocket<{ compute::KIB * 16 }>,
    config: WorldServerConfig,
}

impl WorldServer {
    pub fn new(config: WorldServerConfig) -> Self {
        let world_config = WorldConfig {
            seed: config.seed,
            noise_scale: 0.05,
            simulation_distance: config.simulation_distance,
        };
        Self {
            worlds: vec![Box::new(Earth::new(world_config))],
            players: FxHashMap::default(),
            network: VoxerUdpSocket::bind_port(3100),
            config,
        }
    }

    pub fn tick(&mut self) {
        let mut world_origins = FxHashMap::<usize, Vec<IVec3>>::default();
        for player in self.players.values() {
            world_origins
                .entry(player.location.world)
                .or_default()
                .push(compute::geo::world_to_chunk_pos(player.location.position));
        }
        for (world_index, origins) in world_origins {
            self.worlds[world_index].update_simulated_chunks(&origins);
        }
        for world in self.worlds.iter_mut() {
            world.tick();
        }
    }
    
    pub fn start(&mut self) {
        self.worlds.first_mut().unwrap().start_simulation();
    }
}
