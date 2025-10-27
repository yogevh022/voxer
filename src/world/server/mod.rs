mod session;
mod world;

use crate::compute::MIB;
use crate::voxer_network;
use crate::world::network::{
    MsgChunkData, MsgChunkDataRequest, MsgSetPositionRequest, NetworkHandle, ServerMessage,
    ServerMessageTag,
};
use crate::world::server::session::{ServerPlayerSession, ServerWorldSession};
use crate::world::server::world::{Earth, World};
use crate::world::session::{PlayerLocation, PlayerSession};
use glam::Vec3;
use std::net::SocketAddr;
use voxer_network::NetworkDeserializable;
pub use crate::world::server::world::*;

#[derive(Debug)]
pub struct ServerWorldConfig {
    pub world_config: WorldConfig,
    pub simulation_distance: usize,
}

pub struct ServerWorld {
    config: ServerWorldConfig,
    network: NetworkHandle,
    session: ServerWorldSession,
}

impl ServerWorld {
    pub fn new(config: ServerWorldConfig) -> Self {
        let chunks_size_hint = config.simulation_distance.pow(4); // fixme arbitrary number
        let earth = Box::new(Earth::new(config.world_config.clone(), chunks_size_hint));
        let worlds: Vec<Box<dyn World>> = vec![earth];
        let session = ServerWorldSession::new(worlds);

        let socket_addr = SocketAddr::from(([0, 0, 0, 0], 3100));
        let mut network = NetworkHandle::bind(socket_addr, MIB * 4);
        network.listen();
        Self {
            config,
            network,
            session,
        }
    }

    pub fn start_session(&mut self) {
        self.session.start();
    }

    pub fn tick(&mut self) {
        for message in self.network.take_messages(64) {
            self.handle_network_message(message);
        }
        self.session.tick();
    }

    fn handle_network_message(&mut self, message: ServerMessage) {
        match message.tag {
            ServerMessageTag::ChunkDataRequest => {
                let chunk_req_msg = MsgChunkDataRequest::deserialize(message.message.data);
                let positions = &chunk_req_msg.positions[0..chunk_req_msg.count as usize];
                let chunks = self.session.request_chunks_from_world(0, positions);
                for chunk in chunks {
                    let msg = MsgChunkData {
                        position: chunk.position,
                        blocks: chunk.blocks,
                    };
                    self.network
                        .send_to(Box::new(msg), &message.message.src)
                        .unwrap();
                }
            }
            ServerMessageTag::SetPositionRequest => {
                if self.session.player_by_addr(message.message.src).is_none() {
                    return;
                }
                let position_req = MsgSetPositionRequest::deserialize(message.message.data);
                let player_id = self.session.player_by_addr(message.message.src).unwrap();
                self.session
                    .players
                    .get_mut(&player_id)
                    .unwrap()
                    .player
                    .location
                    .position = position_req.position;
            }
            ServerMessageTag::ConnectRequest => {
                let (pid, paddr) = (0, message.message.src);
                let player = PlayerSession {
                    id: pid,
                    name: "bill".to_string(),
                    location: PlayerLocation {
                        world: 0,
                        position: Vec3::ZERO,
                    },
                };
                let server_player = ServerPlayerSession {
                    player,
                    addr: paddr,
                };
                self.session.add_player(server_player);
            }
            ServerMessageTag::Ping => unimplemented!(),
            _ => unimplemented!(),
        }
    }
}
