mod network;
mod session;

use crate::compute::MIB;
use crate::compute::geo::{Plane, world_to_chunk_pos};
use crate::world::client::network::ClientWorldNetwork;
use crate::world::client::session::ClientWorldSession;
use crate::world::network::{
    MAX_CHUNKS_PER_BATCH, MsgChunkData, MsgChunkDataEmpty, NetworkHandle, ServerMessage,
    ServerMessageTag,
};
use crate::world::server::chunk::VoxelChunk;
use crate::world::session::{PlayerLocation, PlayerSession};
use glam::{IVec3, Vec3};
use std::net::SocketAddr;
use std::sync::Arc;
use voxer_network::NetworkDeserializable;
use wgpu::CommandEncoder;
use winit::window::Window;

#[derive(Clone, Copy)]
pub struct ClientWorldConfig {
    pub render_distance: usize,
}

pub struct ClientWorld<'window> {
    pub config: ClientWorldConfig,
    pub session: ClientWorldSession<'window>,

    player: PlayerSession,

    network: ClientWorldNetwork,
    temp_server_addr: SocketAddr,
}

impl ClientWorld<'_> {
    pub fn new(window: Arc<Window>, config: ClientWorldConfig) -> Self {
        let player = PlayerSession {
            id: 0,
            name: "bill".to_string(),
            location: PlayerLocation {
                world: 0,
                position: Vec3::ZERO,
            },
        };
        let socket_addr = SocketAddr::from(([0, 0, 0, 0], 0));
        let temp_server_addr = SocketAddr::from(([127, 0, 0, 1], 3100));

        let network_handle = NetworkHandle::bind(socket_addr, MIB * 4);
        let mut network = ClientWorldNetwork::new(network_handle);
        network.set_server_addr(temp_server_addr); // fixme temp
        Self {
            config,
            session: ClientWorldSession::new(
                window,
                config,
                world_to_chunk_pos(player.location.position),
            ),
            network,
            player,
            temp_server_addr,
        }
    }

    pub(crate) fn temp_set_player_position(&mut self, position: Vec3) {
        self.player.location.position = position;
    }

    pub(crate) fn temp_set_view_frustum(&mut self, frustum: [Plane; 6]) {
        self.session.view_frustum = frustum;
    }

    pub(crate) fn temp_send_req_conn(&self) {
        self.network.send_connection_request(self.temp_server_addr);
    }

    pub(crate) fn temp_send_player_position(&self) {
        self.network.send_player_position(self.player.location.position);
    }

    fn request_interest_chunks(&mut self, origin: IVec3) {
        self.network.prepare_to_batch_requests();
        self.session
            .missing_interest_chunks(origin, MAX_CHUNKS_PER_BATCH, |p| {
                self.network.batch_chunk_request(p);
            });
        self.network.request_chunk_batch();
    }

    pub fn tick(&mut self, encoder: &mut CommandEncoder) {
        self.network.receive_messages(|msg| {
            Self::handle_network_message(&mut self.session, msg);
        });
        let player_ch_pos = world_to_chunk_pos(self.player.location.position);
        self.session.tick(encoder, player_ch_pos);
        self.request_interest_chunks(player_ch_pos);
    }

    fn handle_network_message(session: &mut ClientWorldSession<'_>, message: ServerMessage) {
        match message.tag {
            ServerMessageTag::ChunkData => {
                let chunk_data_msg = MsgChunkData::deserialize(message.message.data);
                let chunk = VoxelChunk::from(chunk_data_msg);
                session.add_new_chunk(chunk);
            }
            ServerMessageTag::ChunkDataEmpty => {
                let chunk_data_msg = MsgChunkDataEmpty::deserialize(message.message.data);
                let chunk = VoxelChunk::from(chunk_data_msg);
                session.add_new_chunk(chunk);
            }
            ServerMessageTag::SetPosition => {
                todo!()
            }
            ServerMessageTag::Ping => unimplemented!(),
            _ => unimplemented!(),
        }
    }
}
