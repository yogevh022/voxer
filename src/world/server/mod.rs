mod types;
mod world;

use crate::voxer_network::{NetworkSerializable, ReceivedMessage};
use crate::world::generation::WorldConfig;
use crate::world::network::{
    MsgChunkData, MsgChunkDataRequest, NetworkHandle, ServerMessage, ServerMessageTag,
    process_message,
};
use crate::world::server::types::{ServerPlayerSession, ServerWorldSession};
use crate::world::server::world::{Earth, World};
use crate::world::session::{PlayerLocation, PlayerSession};
use crate::{compute, voxer_network};
use glam::Vec3;
use std::net::SocketAddr;
use voxer_network::NetworkDeserializable;

#[derive(Debug)]
pub struct ServerWorldConfig {
    pub seed: i32,
    pub simulation_distance: usize,
}

pub struct ServerWorld {
    config: ServerWorldConfig,
    network: NetworkHandle<{ compute::KIB * 16 }>,
    session: ServerWorldSession,
}

impl ServerWorld {
    pub fn new(config: ServerWorldConfig) -> Self {
        let world_config = WorldConfig {
            seed: config.seed,
            noise_scale: 0.05,
            simulation_distance: config.simulation_distance,
        };
        let worlds: Vec<Box<dyn World>> = vec![Box::new(Earth::new(world_config))];
        let mut session = ServerWorldSession::new(worlds);
        let player = PlayerSession {
            id: 0,
            name: "bill".to_string(),
            location: PlayerLocation {
                world: 0,
                position: Vec3::ZERO,
            },
        };
        let player_session = ServerPlayerSession {
            player,
            addr: SocketAddr::from(([0, 0, 0, 0], 3100)),
        };
        session.add_player(player_session);

        let socket_addr = SocketAddr::from(([0, 0, 0, 0], 3100));
        let mut network = NetworkHandle::bind(socket_addr);
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
        let batch = self
            .network
            .try_iter_messages()
            .take(64)
            .collect::<Vec<_>>();
        for message in batch {
            self.handle_network_message(message);
        }

        self.session.tick();
    }

    fn handle_network_message(&mut self, message: ServerMessage) {
        match message.tag {
            ServerMessageTag::ChunkDataRequest => {
                let chunk_req_msg = MsgChunkDataRequest::deserialize(message.message.data);
                let positions = &chunk_req_msg.positions[0..chunk_req_msg.count as usize];
                let chunks = self.session.get_chunks(0, positions);
                for chunk in chunks {
                    let msg = MsgChunkData {
                        position: chunk.position,
                        solid_count: chunk.solid_count as u32,
                        blocks: chunk.blocks,
                    };
                    self.network
                        .channel
                        .lock()
                        .send_to(Box::new(msg), &message.message.src)
                        .unwrap();
                }
            }
            ServerMessageTag::SetPositionRequest => {
                todo!()
            }
            ServerMessageTag::Ping => unimplemented!(),
            _ => unimplemented!(),
        }
    }
}
