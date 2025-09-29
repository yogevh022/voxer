mod types;

use crate::app::app_renderer;
use crate::app::app_renderer::AppRenderer;
use crate::compute::geo::{AABB, Frustum, Plane};
use crate::world::client::types::ClientWorldSession;
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
use winit::window::Window;
use crate::compute::MIB;

pub struct ClientWorldConfig {
    pub render_distance: usize,
}

pub struct ClientWorld<'window> {
    pub config: ClientWorldConfig,
    pub(crate) renderer: AppRenderer<'window>,
    network: NetworkHandle,
    session: ClientWorldSession,
}

impl<'window> ClientWorld<'window> {
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
        let mut network = NetworkHandle::bind(socket_addr, MIB);
        network.listen();
        Self {
            config,
            renderer: app_renderer::make_app_renderer(window),
            network,
            session: ClientWorldSession::new(player),
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

    pub fn set_view_frustum(&mut self, frustum: [Plane; 6]) {
        self.session.view_frustum = frustum;
    }

    pub fn tick(&mut self) {
        for message in self.network.take_messages(64) {
            self.handle_network_message(message);
        }
        self.session.tick();
    }

    pub fn encode_render_tick(&mut self, encoder: &mut wgpu::CommandEncoder) {
        self.cull_outside_frustum();
        let (missing_chunks, new_renders) = self.nearby_chunks_pass();
        self.renderer.load_chunks(encoder, new_renders);
        self.request_chunks(&missing_chunks);
    }

    fn cull_outside_frustum(&mut self) {
        let mut frustum_aabb = Frustum::aabb(&self.session.view_frustum);
        frustum_aabb.min = (frustum_aabb.min / CHUNK_DIM as f32).floor();
        frustum_aabb.max = (frustum_aabb.max / CHUNK_DIM as f32).ceil();

        self.renderer.retain_chunk_positions(|c_pos| {
            let min = c_pos.as_vec3();
            let chunk_aabb = AABB { min, max: min + 1.0 };
            AABB::within_aabb(chunk_aabb, frustum_aabb)
        });
    }

    fn nearby_chunks_pass(&self) -> (Vec<IVec3>, Vec<Chunk>) {
        let mut missing_positions = Vec::with_capacity(MAX_CHUNKS_PER_BATCH);
        let mut new_render = Vec::with_capacity(1024); // fixme arbitrary number

        let mut frustum_aabb = Frustum::aabb(&self.session.view_frustum);
        frustum_aabb.min = (frustum_aabb.min / CHUNK_DIM as f32).floor();
        frustum_aabb.max = (frustum_aabb.max / CHUNK_DIM as f32).ceil();

        frustum_aabb.discrete_points(|chunk_position| {
            match self.session.chunks.get(&chunk_position) {
                Some(chunk) if !self.renderer.is_chunk_rendered(chunk_position) => {
                    new_render.push(chunk.clone())
                }
                None if missing_positions.len() < MAX_CHUNKS_PER_BATCH => {
                    missing_positions.push(chunk_position)
                }
                _ => {}
            }
        });

        (missing_positions, new_render)
    }

    fn request_chunks(&mut self, positions: &[IVec3]) {
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
