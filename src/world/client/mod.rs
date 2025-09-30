mod session;
use crate::compute::MIB;
use crate::compute::geo::{Frustum, Plane};
use crate::world::client::session::ClientWorldSession;
use crate::world::network::{
    MAX_CHUNKS_PER_BATCH, MsgChunkData, MsgChunkDataRequest, MsgConnectRequest,
    MsgSetPositionRequest, NetworkHandle, ServerMessage, ServerMessageTag,
};
use crate::world::session::{PlayerLocation, PlayerSession};
use crate::world::types::{CHUNK_DIM, Chunk};
use glam::{IVec3, Vec3};
use std::net::SocketAddr;
use std::sync::Arc;
use voxer_network::NetworkDeserializable;
use wgpu::CommandEncoder;
use winit::window::Window;

pub struct ClientWorldConfig {
    pub render_distance: usize,
}

pub struct ClientWorld<'window> {
    pub config: ClientWorldConfig,
    pub session: ClientWorldSession<'window>,
    network: NetworkHandle,
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
        Self {
            config,
            network,
            session: ClientWorldSession::new(window, player),
        }
    }

    pub fn temp_set_player_position(&mut self, position: Vec3) {
        self.session.player.location.position = position;
    }

    pub fn temp_send_player_position(&self) {
        let temp_server_addr = SocketAddr::from(([127, 0, 0, 1], 3100));
        let pos_req = MsgSetPositionRequest {
            position: self.session.player.location.position,
        };
        self.network
            .send_to(Box::new(pos_req), &temp_server_addr)
            .unwrap();
    }

    pub fn temp_send_req_conn(&self) {
        let temp_server_addr = SocketAddr::from(([127, 0, 0, 1], 3100));
        let con_req = MsgConnectRequest { byte: 62 };
        self.network
            .send_to(Box::new(con_req), &temp_server_addr)
            .unwrap();
    }

    pub fn temp_set_view_frustum(&mut self, frustum: [Plane; 6]) {
        self.session.view_frustum = frustum;
    }

    pub fn tick(&mut self, gpu_encoder: &mut CommandEncoder) {
        for message in self.network.take_messages(64) {
            self.handle_network_message(message);
        }
        self.session.tick();
        self.session.update_render_state(gpu_encoder);
        self.request_chunk_batch();
    }

    fn request_chunk_batch(&mut self) {
        let positions = self.session.chunk_request_batch();
        debug_assert!(positions.len() <= MAX_CHUNKS_PER_BATCH);
        if positions.is_empty() {
            return;
        }
        let temp_server_addr = SocketAddr::from(([127, 0, 0, 1], 3100)); // temp
        let mut arr = [IVec3::ZERO; MAX_CHUNKS_PER_BATCH];
        let positions_capped = &positions[0..positions.len()];
        arr[0..positions_capped.len()].copy_from_slice(positions_capped);
        let msg = MsgChunkDataRequest::new(positions_capped.len() as u8, arr);

        self.network
            .send_to(Box::new(msg), &temp_server_addr)
            .unwrap();
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
                self.session.add_chunk(chunk);
            }
            ServerMessageTag::SetPosition => {
                todo!()
            }
            ServerMessageTag::Ping => unimplemented!(),
            _ => unimplemented!(),
        }
    }
}
