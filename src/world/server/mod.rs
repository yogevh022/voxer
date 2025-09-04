mod config;
mod player;
mod world;

pub use config::WorldServerConfig;
use std::net::SocketAddr;

use crate::voxer_network::ReceivedMessage;
use crate::world::generation::WorldConfig;
use crate::world::network::{ServerMessageTag, process_message};
use crate::world::server::player::Player;
use crate::world::server::world::{Earth, World};
use crate::{compute, voxer_network};
use glam::IVec3;
use rustc_hash::FxHashMap;

pub struct WorldServer {
    worlds: Vec<Box<dyn World>>,
    players: FxHashMap<usize, Player>,
    network: voxer_network::UdpChannel<{ compute::KIB * 16 }>,
    config: WorldServerConfig,
}

impl WorldServer {
    pub fn new(config: WorldServerConfig) -> Self {
        let world_config = WorldConfig {
            seed: config.seed,
            noise_scale: 0.05,
            simulation_distance: config.simulation_distance,
        };
        let socket_addr = SocketAddr::from(([0, 0, 0, 0], 3100));
        Self {
            worlds: vec![Box::new(Earth::new(world_config))],
            players: FxHashMap::default(),
            network: voxer_network::UdpChannel::bind(socket_addr),
            config,
        }
    }

    pub fn tick(&mut self) {
        let network_messages = self.network.recv_complete();
        for message in network_messages {
            self.handle_network_message(message);
        }
        
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

    fn handle_network_message(&mut self, message: ReceivedMessage) {
        let server_message = process_message(message);
        match server_message.tag {
            ServerMessageTag::Ping => {
                println!(
                    "Server: Ping received! {}",
                    server_message.message.data.get(0).copied().unwrap()
                );
            }
            ServerMessageTag::ChunkDataRequest => {
                todo!()
            }
            ServerMessageTag::SetPositionRequest => {
                todo!()
            }
            ServerMessageTag::ChunkData => {
                todo!()
            }
            ServerMessageTag::SetPosition => {
                todo!()
            }
            _ => unimplemented!(),
        }
    }
}
