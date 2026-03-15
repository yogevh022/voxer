mod network;
mod session;

use crate::app::app_renderer::AppRenderer;
use crate::compute::MIB;
use crate::compute::geo::{Frustum, Plane, world_to_chunk_pos};
use crate::vtypes::Camera;
use crate::world::CHUNK_DIM;
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
    pub session: ClientWorldSession,
    pub renderer: AppRenderer<'window>,

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
        let mut network = ClientWorldNetwork::new(network_handle, config.render_distance as u32);
        network.set_server_addr(temp_server_addr); // fixme temp
        Self {
            config,
            session: ClientWorldSession::new(config, world_to_chunk_pos(player.location.position)),
            renderer: AppRenderer::new(window, config.render_distance),
            network,
            player,
            temp_server_addr,
        }
    }

    pub(crate) fn temp_set_camera(&mut self, camera: Camera) {
        self.player.location.position = camera.transform.position;
        self.session.camera = camera;
    }

    pub(crate) fn temp_send_req_conn(&self) {
        self.network.send_connection_request(self.temp_server_addr);
    }

    pub(crate) fn temp_send_player_position(&self) {
        self.network
            .send_player_position(self.player.location.position);
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
        self.session.tick(player_ch_pos);

        // render fixme move logic to renderer
        let safe_voxel_rdist = ((self.config.render_distance - 1) * CHUNK_DIM) as f32;
        let safe_culling_vp = self.session.camera.projection_with_far(safe_voxel_rdist)
            * self.session.camera.view_matrix();
        let view_planes = Frustum::planes(safe_culling_vp);

        let mesh_chunks = self.session.chunk_meshing_batch();
        self.renderer
            .update_chunk_meshes(encoder, mesh_chunks, player_ch_pos, &view_planes);

        self.request_interest_chunks(player_ch_pos);
    }

    fn handle_network_message(session: &mut ClientWorldSession, message: ServerMessage) {
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
