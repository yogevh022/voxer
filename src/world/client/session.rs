use crate::app::app_renderer::AppRenderer;
use crate::compute::geo::{Plane, ivec3_with_adjacent_positions, world_to_chunk_pos};
use crate::compute::throttler::Throttler;
use crate::world::ClientWorldConfig;
use crate::world::network::MAX_CHUNKS_PER_BATCH;
use crate::world::session::PlayerSession;
use glam::IVec3;
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use wgpu::{CommandEncoder, ComputePassDescriptor};
use winit::window::Window;
use crate::world::server::chunk::VoxelChunk;

pub struct ClientWorldSession<'window> {
    pub player: PlayerSession,
    pub app_renderer: AppRenderer<'window>,
    pub view_frustum: [Plane; 6],
    pub chunks: FxHashMap<IVec3, VoxelChunk>,

    render_max_sq: i32,

    chunk_request_throttler: Throttler,
    chunk_request_batch: Vec<IVec3>,

    lazy_chunk_positions: Vec<IVec3>,

    chunk_meshing_batch: FxHashSet<IVec3>,
}

impl<'window> ClientWorldSession<'window> {
    pub fn new(window: Arc<Window>, player: PlayerSession, config: ClientWorldConfig) -> Self {
        let mut chunks = FxHashMap::default();
        chunks.reserve((config.render_distance * 2).pow(3));
        Self {
            player,
            app_renderer: AppRenderer::new(window),
            view_frustum: [Plane::default(); 6],
            chunks,
            render_max_sq: (config.render_distance as i32).pow(2),
            chunk_request_throttler: Throttler::new((1 << 18) + 1, Duration::from_millis(200)),
            chunk_request_batch: Vec::new(),
            lazy_chunk_positions: Vec::new(),
            chunk_meshing_batch: FxHashSet::default(),
        }
    }

    pub fn add_new_chunk(&mut self, chunk: VoxelChunk) {
        self.chunk_meshing_batch
            .extend(ivec3_with_adjacent_positions(chunk.position));
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
                self.app_renderer.chunk_session.drop_chunk(&position);
                self.chunks.remove(&position);
            }
        }
    }

    pub fn tick(&mut self, encoder: &mut CommandEncoder) {
        let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Client Compute Pass"),
            timestamp_writes: None,
        });
        self.app_renderer
            .renderer
            .depth
            .generate_depth_mips(&self.app_renderer.renderer.device, &mut compute_pass);

        let player_ch_position = world_to_chunk_pos(self.player.location.position);
        self.lazy_chunk_gc(player_ch_position);

        let max_write = self.app_renderer.chunk_session.config.max_write_count;
        let mesh_chunks_refs = self
            .chunk_meshing_batch
            .drain()
            .filter_map(|p| self.chunks.get(&p))
            .take(max_write);

        self.app_renderer
            .chunk_session
            .prepare_chunk_writes(mesh_chunks_refs);

        self.chunk_request_batch.clear();
        self.chunk_request_throttler.set_now(Instant::now());
        self.app_renderer
            .chunk_session
            .prepare_chunk_visibility(&self.view_frustum, |ch_pos| {
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

        self.app_renderer
            .chunk_session
            .compute_chunk_writes(&self.app_renderer.renderer, &mut compute_pass);
        self.app_renderer
            .chunk_session
            .compute_chunk_visibility_and_meshing(&self.app_renderer.renderer, &mut compute_pass);
    }
}
