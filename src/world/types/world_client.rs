use std::net::SocketAddr;
use crate::app::app_renderer;
use crate::app::app_renderer::AppRenderer;
use crate::compute;
use crate::compute::array::Array3D;
use crate::compute::geo;
use crate::world::types::chunk::ChunkAdjacentBlocks;
use crate::world::types::{VoxelBlock, CHUNK_DIM, Chunk};
use glam::{IVec3, Vec3};
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::Arc;
use winit::window::Window;
use crate::voxer_network::UdpChannel;
use crate::world::network;

pub struct WorldClientConfig {
    pub render_distance: usize,
}

pub struct WorldClient<'window> {
    pub config: WorldClientConfig,
    pub renderer: AppRenderer<'window, 2, 1>,
    pub chunks: FxHashMap<IVec3, Chunk>,
    network: UdpChannel<1024>,
    player_position: Vec3,
}

impl<'window> WorldClient<'window> {
    pub fn new(window: Arc<Window>, config: WorldClientConfig) -> Self {
        let socket_addr = SocketAddr::from(([0, 0, 0, 0], 0));
        Self {
            renderer: app_renderer::make_app_renderer(window),
            chunks: FxHashMap::default(),
            config,
            network: UdpChannel::bind(socket_addr),
            player_position: Vec3::ZERO,
        }
    }

    pub fn ping_server(&self) {
        let server_addr = SocketAddr::from(([127, 0, 0, 1], 3100));
        let ping = network::MsgPing {
            byte: 62,
        };
        let ping_res = self.network.send_to(ping, &server_addr);
        println!("Client: {:?} -> {:?}", ping.byte, ping_res);
    }

    pub fn add_chunks(&mut self, chunks: Vec<Chunk>) {
        let mut new_positions = Vec::with_capacity(chunks.len());
        for chunk in chunks {
            let c_pos = chunk.position;
            self.chunks.insert(chunk.position, chunk);
            new_positions.push(c_pos);
        }
        new_positions.into_iter().for_each(|chunk_position| {
            // todo fix redundancy in repeated calculation
            self.update_chunk_data(chunk_position);
            self.update_adjacent_chunks_data(chunk_position);
        });
    }

    pub fn remove_chunks(&mut self, positions: Vec<IVec3>) {
        for pos in positions {
            self.chunks.remove(&pos);
        }
    }

    fn update_chunk_data(&mut self, position: IVec3) {
        let adjacent_blocks = Array3D(compute::chunk::get_adjacent_blocks(position, &self.chunks));
        self.chunks.get_mut(&position).map(|chunk| {
            chunk.face_count = Some(compute::chunk::face_count(&chunk.blocks, &adjacent_blocks));
            chunk.adjacent_blocks = adjacent_blocks;
        });
    }

    fn update_adjacent_chunks_data(&mut self, origin_position: IVec3) {
        for chunk_position in [
            IVec3::new(origin_position.x - 1, origin_position.y, origin_position.z),
            IVec3::new(origin_position.x, origin_position.y - 1, origin_position.z),
            IVec3::new(origin_position.x, origin_position.y, origin_position.z - 1),
        ] {
            self.update_chunk_data(chunk_position);
        }
    }

    pub fn chunks_to_mesh(&self, frustum_planes: &[geo::Plane; 6]) -> Vec<IVec3> {
        let mut added_chunks = FxHashSet::default();
        let mut positions = Vec::new();

        geo::Sphere::discrete_points(
            geo::world_to_chunk_pos(self.player_position),
            self.config.render_distance as isize,
            |chunk_position| {
                // todo optimize this
                let chunk_world_position = geo::chunk_to_world_pos(chunk_position);
                if geo::Frustum::is_aabb_within_frustum(
                    chunk_world_position,
                    chunk_world_position + CHUNK_DIM as f32,
                    &frustum_planes,
                ) && !self.renderer.is_chunk_rendered(chunk_position)
                {
                    added_chunks.insert(chunk_position);
                    positions.push(chunk_position);
                    let mx_chunk_position =
                        IVec3::new(chunk_position.x - 1, chunk_position.y, chunk_position.z);
                    let my_chunk_position =
                        IVec3::new(chunk_position.x, chunk_position.y - 1, chunk_position.z);
                    let mz_chunk_position =
                        IVec3::new(chunk_position.x, chunk_position.y, chunk_position.z - 1);
                    if added_chunks.insert(mx_chunk_position) {
                        positions.push(mx_chunk_position);
                    }
                    if added_chunks.insert(my_chunk_position) {
                        positions.push(my_chunk_position);
                    }
                    if added_chunks.insert(mz_chunk_position) {
                        positions.push(mz_chunk_position);
                    }
                };
            },
        );
        positions
    }

    pub fn get_chunks(&self, positions: Vec<IVec3>) -> Vec<Chunk> {
        positions
            .into_iter()
            .filter_map(|pos| self.chunks.get(&pos).cloned())
            .collect()
    }

    pub fn set_player_position(&mut self, position: Vec3) {
        self.player_position = position;
    }
}
