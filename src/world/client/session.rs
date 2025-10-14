use crate::app::app_renderer::AppRenderer;
use crate::compute;
use crate::compute::array::Array3D;
use crate::compute::geo::{
    AABB, Frustum, Plane, chunk_to_world_pos, ivec3_with_adjacent_positions, world_to_chunk_pos,
};
use crate::compute::throttler::Throttler;
use crate::world::ClientWorldConfig;
use crate::world::network::MAX_CHUNKS_PER_BATCH;
use crate::world::session::PlayerSession;
use crate::world::types::{CHUNK_DIM, Chunk};
use glam::IVec3;
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use wgpu::{CommandEncoder, ComputePassDescriptor};
use winit::window::Window;

pub struct ClientWorldSession<'window> {
    pub player: PlayerSession,
    pub renderer: AppRenderer<'window>,
    pub view_frustum: [Plane; 6],
    pub chunks: FxHashMap<IVec3, Chunk>,

    render_max_sq: i32,

    chunk_request_throttler: Throttler,
    chunk_request_batch: Vec<IVec3>,
    chunk_render_batch: Vec<Chunk>,

    lazy_chunk_positions: Vec<IVec3>,

    new_chunks: Vec<IVec3>,
}

impl<'window> ClientWorldSession<'window> {
    pub fn new(window: Arc<Window>, player: PlayerSession, config: ClientWorldConfig) -> Self {
        let mut chunks = FxHashMap::default();
        chunks.reserve(config.render_distance.pow(3));
        Self {
            player,
            renderer: AppRenderer::new(window),
            view_frustum: [Plane::default(); 6],
            chunks,
            render_max_sq: (config.render_distance as i32).pow(2),
            chunk_request_throttler: Throttler::new((1 << 18) + 1, Duration::from_millis(200)),
            chunk_request_batch: Vec::new(),
            chunk_render_batch: Vec::new(),
            lazy_chunk_positions: Vec::new(),
            new_chunks: Vec::new(),
        }
    }

    pub fn add_new_chunk(&mut self, chunk: Chunk) {
        self.new_chunks.push(chunk.position);
        self.chunks.insert(chunk.position, chunk);
    }

    pub fn chunk_request_batch(&self) -> &[IVec3] {
        &self.chunk_request_batch
    }

    pub fn lazy_chunk_gc(&mut self, camera_ch_position: IVec3) {
        if self.lazy_chunk_positions.is_empty() {
            self.lazy_chunk_positions.extend(self.chunks.keys());
            return;
        }
        const CHUNKS_PER_PASS: usize = 32; // fixme add to centralized config
        let remaining_positions = self.lazy_chunk_positions.len();
        for position in self
            .lazy_chunk_positions
            .drain(remaining_positions.saturating_sub(CHUNKS_PER_PASS)..)
        {
            if camera_ch_position.distance_squared(position) > self.render_max_sq {
                self.renderer.chunk_manager.drop_chunk(&position);
                self.chunks.remove(&position);
            }
        }
    }

    pub fn tick(&mut self, encoder: &mut CommandEncoder) {
        // fixme temp
        // fixme no enforcement of max_new_chunks

        let player_ch_position = world_to_chunk_pos(self.player.location.position);
        self.lazy_chunk_gc(player_ch_position);

        let to_remesh = self.process_chunk_remesh_batch();

        let new_chunks = self
            .new_chunks
            .drain(..)
            .chain(to_remesh.into_iter())
            .map(|p| self.chunks.get(&p).unwrap().clone())
            .collect::<Vec<_>>();

        self.renderer.chunk_manager.update_chunk_writes(&new_chunks);

        self.chunk_request_batch.clear();
        self.chunk_request_throttler.set_now(Instant::now());

        self.renderer
            .chunk_manager
            .update_view_chunks(&self.view_frustum, |ch_pos| {
                if player_ch_position.distance_squared(ch_pos) > self.render_max_sq {
                    return;
                }
                if self.chunk_request_batch.len() < MAX_CHUNKS_PER_BATCH {
                    let throttle_idx = smallhash::u32x3_to_18_bits(ch_pos.to_array());
                    if self.chunk_request_throttler.request(throttle_idx as usize) {
                        self.chunk_request_batch.push(ch_pos);
                    }
                }
            });

        let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Chunk Compute Pass"),
            timestamp_writes: None,
        });
        self.renderer
            .chunk_manager
            .encode_gpu_chunk_writes(&self.renderer.renderer, &mut compute_pass);
        self.renderer
            .chunk_manager
            .encode_gpu_view_chunks(&self.renderer.renderer, &mut compute_pass);
    }

    fn process_chunk_remesh_batch(&mut self) -> FxHashSet<IVec3> {
        let mut positions: FxHashSet<_> = self
            .new_chunks
            .drain(..)
            .map(|p| ivec3_with_adjacent_positions(p))
            .flatten()
            .collect();
        positions.retain(|p| self.update_chunk_mesh_data(*p));
        positions
    }

    fn update_chunk_mesh_data(&mut self, position: IVec3) -> bool {
        let adj_blocks = Array3D(compute::chunk::get_adj_blocks(position, &self.chunks));
        if let Some(chunk) = self.chunks.get_mut(&position) {
            chunk.face_count = Some(compute::chunk::face_count(&chunk.blocks, &adj_blocks));
            chunk.adjacent_blocks = adj_blocks;
            return true;
        }
        false
    }
}
