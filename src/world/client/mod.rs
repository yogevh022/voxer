mod types;

use crate::app::app_renderer;
use crate::app::app_renderer::AppRenderer;
use crate::compute;
use crate::compute::geo::{Frustum, Plane, AABB};
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
use std::time::Instant;
use voxer_network::NetworkDeserializable;
use winit::window::Window;

pub struct ClientWorldConfig {
    pub render_distance: usize,
}

pub struct ClientWorld<'window> {
    pub config: ClientWorldConfig,
    pub(crate) renderer: AppRenderer<'window>,
    network: NetworkHandle<{ compute::KIB * 16 }>,
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
        let mut network = NetworkHandle::<{ compute::KIB * 16 }>::bind(socket_addr);
        network.listen();
        Self {
            config,
            renderer: app_renderer::make_app_renderer(window),
            network,
            session: ClientWorldSession::new(player),
        }
    }

    pub fn set_player_position(&mut self, position: Vec3) {
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
        let batch = self
            .network
            .try_iter_messages()
            .take(64)
            .collect::<Vec<_>>();
        for message in batch {
            self.handle_network_message(message);
        }
        self.session.tick();

        // render
        self.cull_outside_frustum();
        let (missing_chunks, new_renders) = self.nearby_chunks_pass();
        self.renderer.load_chunks(
            &mut new_renders
                .iter()
                .map(|pos| self.session.chunks.get(pos).unwrap()),
        );
        self.session.missing_chunks = Some(missing_chunks);
    }

    fn cull_outside_frustum(&mut self) {
        let mut frustum_aabb = Frustum::aabb(&self.session.view_frustum);
        frustum_aabb.min = (frustum_aabb.min / CHUNK_DIM as f32).floor();
        frustum_aabb.max = (frustum_aabb.max / CHUNK_DIM as f32).ceil();

        self.renderer.retain_chunk_positions(|c_pos| {
            let min = c_pos.as_vec3();
            let max = min + 1f32;
            let c_aabb = AABB { min, max };
            AABB::within_aabb(c_aabb, frustum_aabb)
        });
    }

    fn nearby_chunks_pass(&mut self) -> (Vec<IVec3>, Vec<IVec3>) {
        let mut missing_positions = Vec::with_capacity(MAX_CHUNKS_PER_BATCH);
        let mut new_render = Vec::new();

        let mut frustum_aabb = Frustum::aabb(&self.session.view_frustum);
        frustum_aabb.min = (frustum_aabb.min / CHUNK_DIM as f32).floor();
        frustum_aabb.max = (frustum_aabb.max / CHUNK_DIM as f32).ceil();

        let now = Instant::now();

        frustum_aabb.discrete_points(|chunk_position| {
            if !self.session.chunks.contains_key(&chunk_position) {
                if missing_positions.len() < MAX_CHUNKS_PER_BATCH
                    && self.session.try_request_permission(now, chunk_position)
                {
                    missing_positions.push(chunk_position);
                }
            } else if !self.renderer.is_chunk_rendered(chunk_position) {
                new_render.push(chunk_position);
            }
        });

        (missing_positions, new_render)
    }

    pub fn request_missing_chunks(&mut self) {
        let can_request = self.session.missing_chunks.take().unwrap_or_default();
        if can_request.is_empty() {
            return;
        }
        let temp_server_addr = SocketAddr::from(([127, 0, 0, 1], 3100)); // temp
        let mut arr = [IVec3::ZERO; MAX_CHUNKS_PER_BATCH];
        let positions_capped = &can_request[0..can_request.len()];
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
