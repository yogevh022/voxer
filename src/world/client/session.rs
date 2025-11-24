use crate::app::app_renderer::AppRenderer;
use crate::compute::geo::{
    Circle, Plane, Sphere, ivec3_with_adjacent_positions, world_to_chunk_pos,
};
use crate::compute::throttler::Throttler;
use crate::world::ClientWorldConfig;
use crate::world::network::MAX_CHUNKS_PER_BATCH;
use crate::world::server::chunk::VoxelChunk;
use crate::world::session::PlayerSession;
use glam::IVec3;
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use wgpu::{CommandEncoder, ComputePassDescriptor};
use winit::window::Window;

pub struct ClientWorldSession<'window> {
    pub player: PlayerSession,
    pub app_renderer: AppRenderer<'window>,
    pub view_frustum: [Plane; 6],
    pub chunks: FxHashMap<IVec3, VoxelChunk>,

    render_max: i32,
    render_max_sq: i32,

    chunk_request_throttler: Throttler,
    chunk_request_batch: Vec<IVec3>,

    chunk_gc_batch: Vec<IVec3>,
    chunk_nearby_position_circles: Vec<(IVec3, isize)>,
    chunk_nearby_positions: Vec<IVec3>,

    chunk_meshing_batch: FxHashSet<IVec3>,
}

impl<'window> ClientWorldSession<'window> {
    pub fn new(window: Arc<Window>, player: PlayerSession, config: ClientWorldConfig) -> Self {
        let mut chunks = FxHashMap::default();
        chunks.reserve((config.render_distance * 2).pow(3));
        Self {
            player,
            app_renderer: AppRenderer::new(window, config.render_distance),
            view_frustum: [Plane::default(); 6],
            chunks,
            render_max: config.render_distance as i32, // fixme config
            render_max_sq: (config.render_distance as i32).pow(2),
            chunk_request_throttler: Throttler::new((1 << 18) + 1, Duration::from_millis(200)),
            chunk_request_batch: Vec::new(),
            chunk_gc_batch: Vec::new(),
            chunk_nearby_position_circles: Vec::new(),
            chunk_nearby_positions: Vec::new(), // fixme
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

    pub fn chunk_gc_pass(&mut self, camera_ch_position: IVec3) {
        if self.chunk_gc_batch.is_empty() {
            self.chunk_gc_batch.extend(self.chunks.keys());
        }
        const CHUNKS_PER_PASS: usize = 32; // fixme add to centralized config
        let position_range = self.chunk_gc_batch.len().saturating_sub(CHUNKS_PER_PASS)..;
        for position in self.chunk_gc_batch.drain(position_range) {
            if camera_ch_position.distance_squared(position) > self.render_max_sq {
                self.chunks.remove(&position);
            }
        }
    }

    pub fn chunk_request_pass(&mut self) {
        // fixme optimize this
        if self.chunk_nearby_position_circles.is_empty() {
            let origin_chunk_pos = world_to_chunk_pos(self.player.location.position);
            let radius = self.render_max as isize - 1;
            Sphere::circles_on_z(
                origin_chunk_pos,
                radius,
                |circle_position, circle_radius| {
                    self.chunk_nearby_position_circles
                        .push((circle_position, circle_radius));
                },
            );
        }
        if self.chunk_nearby_positions.is_empty() {
            let (pos, rad) = self.chunk_nearby_position_circles.pop().unwrap();
            Circle::discrete_points(pos.truncate(), rad, |x, y| {
                let ch_pos = IVec3::new(x as i32, y as i32, pos.z);
                if !self.chunks.contains_key(&ch_pos) {
                    self.chunk_nearby_positions
                        .push(IVec3::new(x as i32, y as i32, pos.z));
                }
            });
        }
        self.chunk_request_batch.clear();
        self.chunk_request_throttler.set_now(Instant::now());
        let position_range = self
            .chunk_nearby_positions
            .len()
            .saturating_sub(MAX_CHUNKS_PER_BATCH)..;
        for ch_pos in self.chunk_nearby_positions.drain(position_range) {
            let throttle_idx = smallhash::u32x3_to_18_bits(ch_pos.to_array());
            if self.chunk_request_throttler.request(throttle_idx as usize) {
                self.chunk_request_batch.push(ch_pos);
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

        self.app_renderer
            .chunk_session
            .set_view_box(&self.view_frustum);

        let player_ch_position = world_to_chunk_pos(self.player.location.position);
        self.chunk_gc_pass(player_ch_position);
        self.chunk_request_pass();

        let max_write = self.app_renderer.chunk_session.config.max_write_count;
        let mesh_chunks_refs = self
            .chunk_meshing_batch
            .drain()
            .filter_map(|p| self.chunks.get(&p))
            .take(max_write);

        self.app_renderer
            .chunk_session
            .prepare_chunk_writes(mesh_chunks_refs);
        self.app_renderer.chunk_session.prepare_chunk_visibility();

        self.app_renderer
            .chunk_session
            .compute_chunk_writes(&self.app_renderer.renderer, &mut compute_pass);
        self.app_renderer
            .chunk_session
            .compute_chunk_visibility_and_meshing(&self.app_renderer.renderer, &mut compute_pass);
    }
}
