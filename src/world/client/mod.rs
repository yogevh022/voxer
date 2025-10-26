mod session;
use crate::compute::MIB;
use crate::compute::geo::Plane;
use crate::world::client::session::ClientWorldSession;
use crate::world::network::{
    MsgChunkData, MsgChunkDataRequest, MsgConnectRequest, MsgSetPositionRequest, NetworkHandle,
    ServerMessage, ServerMessageTag,
};
use crate::world::session::{PlayerLocation, PlayerSession};
use crate::world::types::Chunk;
use glam::Vec3;
use std::net::SocketAddr;
use std::sync::Arc;
use voxer_network::NetworkDeserializable;
use wgpu::{CommandEncoder, ComputePass};
use winit::window::Window;

#[derive(Clone, Copy)]
pub struct ClientWorldConfig {
    pub render_distance: usize,
}

pub struct ClientWorld<'window> {
    pub config: ClientWorldConfig,
    pub session: ClientWorldSession<'window>,
    network: NetworkHandle,
    temp_server_addr: SocketAddr,
}

impl ClientWorld<'_> {
    pub fn new(window: Arc<Window>, config: ClientWorldConfig) -> Self {
        let socket_addr = SocketAddr::from(([0, 0, 0, 0], 0));
        let player = PlayerSession {
            id: 0,
            name: "bill".to_string(),
            location: PlayerLocation {
                world: 0,
                position: Vec3::ZERO,
            },
        };
        let mut network = NetworkHandle::bind(socket_addr, MIB * 4);
        network.listen();
        let temp_server_addr = SocketAddr::from(([127, 0, 0, 1], 3100));
        Self {
            config,
            session: ClientWorldSession::new(window, player, config),
            network,
            temp_server_addr,
        }
    }

    pub fn temp_set_player_position(&mut self, position: Vec3) {
        self.session.player.location.position = position;
    }

    pub fn temp_send_player_position(&self) {
        let set_position_request = MsgSetPositionRequest {
            position: self.session.player.location.position,
        };
        let msg = Box::new(set_position_request);
        self.network.send_to(msg, &self.temp_server_addr).unwrap();
    }

    pub fn temp_send_req_conn(&self) {
        let connection_request = MsgConnectRequest { byte: 62 };
        let msg = Box::new(connection_request);
        self.network.send_to(msg, &self.temp_server_addr).unwrap();
    }

    pub fn temp_set_view_frustum(&mut self, frustum: [Plane; 6]) {
        self.session.view_frustum = frustum;
    }

    pub fn tick(&mut self, encoder: &mut CommandEncoder) {
        for message in self.network.take_messages(64) {
            self.handle_network_message(message);
        }
        self.session.tick(encoder);
        self.request_chunk_batch();
    }

    fn request_chunk_batch(&mut self) {
        let positions = self.session.chunk_request_batch();
        if positions.is_empty() {
            return;
        }
        let chunk_data_request = MsgChunkDataRequest::new_with_positions(positions);
        let msg = Box::new(chunk_data_request);
        self.network.send_to(msg, &self.temp_server_addr).unwrap();
    }

    fn handle_network_message(&mut self, message: ServerMessage) {
        match message.tag {
            ServerMessageTag::ChunkData => {
                let chunk_data_msg = MsgChunkData::deserialize(message.message.data);
                let chunk = Chunk::new(
                    chunk_data_msg.position,
                    chunk_data_msg.blocks,
                    chunk_data_msg.solid_count as usize,
                );
                self.session.add_new_chunk(chunk);
            }
            ServerMessageTag::SetPosition => {
                todo!()
            }
            ServerMessageTag::Ping => unimplemented!(),
            _ => unimplemented!(),
        }
    }
}
