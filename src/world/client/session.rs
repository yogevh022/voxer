use crate::app::app_renderer::AppRenderer;
use crate::compute;
use crate::compute::array::Array3D;
use crate::compute::geo::{AABB, Frustum, Plane, world_to_chunk_pos};
use crate::compute::throttler::Throttler;
use crate::world::ClientWorldConfig;
use crate::world::network::MAX_CHUNKS_PER_BATCH;
use crate::world::session::PlayerSession;
use crate::world::types::{CHUNK_DIM, Chunk};
use glam::IVec3;
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use wgpu::CommandEncoder;
use winit::window::Window;

pub struct ClientWorldSession<'window> {
    pub player: PlayerSession,
    pub config: ClientWorldConfig,
    pub renderer: AppRenderer<'window>,
    pub view_frustum: [Plane; 6],
    pub chunks: FxHashMap<IVec3, Chunk>,
    unprocessed_chunk_positions: Vec<IVec3>,

    chunk_request_throttler: Throttler,
    chunk_request_batch: Vec<IVec3>,
    chunk_render_batch: Vec<Chunk>,

    lazy_chunk_positions: Vec<IVec3>,
}

impl<'window> ClientWorldSession<'window> {
    pub fn new(window: Arc<Window>, player: PlayerSession, config: ClientWorldConfig) -> Self {
        let mut chunks = FxHashMap::default();
        chunks.reserve(config.render_distance.pow(3));
        Self {
            player,
            config, // fixme config redundancy
            renderer: AppRenderer::new(window),
            view_frustum: [Plane::default(); 6],
            chunks,
            unprocessed_chunk_positions: Vec::new(),

            chunk_request_throttler: Throttler::new((1 << 18) + 1, Duration::from_millis(200)),
            chunk_request_batch: Vec::new(),
            chunk_render_batch: Vec::new(),
            lazy_chunk_positions: Vec::new(),
        }
    }

    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.unprocessed_chunk_positions.push(chunk.position);
        self.chunks.insert(chunk.position, chunk);
    }

    pub fn chunk_request_batch(&self) -> &[IVec3] {
        &self.chunk_request_batch
    }

    pub fn lazy_chunk_gc(&mut self) {
        if self.lazy_chunk_positions.is_empty() {
            self.lazy_chunk_positions.extend(self.chunks.keys());
            return;
        }
        const CHUNKS_PER_PASS: usize = 32;
        let remaining_positions = self.lazy_chunk_positions.len();
        let render_threshold_sq = (self.config.render_distance as i32).pow(2) + 1;
        let camera_chunk_position = world_to_chunk_pos(self.player.location.position);
        for position in self
            .lazy_chunk_positions
            .drain(remaining_positions.saturating_sub(CHUNKS_PER_PASS)..)
        {
            if camera_chunk_position.distance_squared(position) > render_threshold_sq {
                self.chunks.remove(&position);
            }
        }
    }

    pub fn update_render_state(&mut self, encoder: &mut CommandEncoder) {
        let mut frustum_aabb = Frustum::aabb(&self.view_frustum);
        frustum_aabb.min = (frustum_aabb.min / CHUNK_DIM as f32).floor();
        frustum_aabb.max = (frustum_aabb.max / CHUNK_DIM as f32).ceil();

        self.retain_frustum_chunks(frustum_aabb);
        self.update_chunk_batches(frustum_aabb);
        self.lazy_chunk_gc();
        self.renderer
            .encode_new_chunks(encoder, &self.chunk_render_batch);
    }

    fn update_chunk_batches(&mut self, frustum_aabb: AABB) {
        self.chunk_request_batch.clear();
        self.chunk_render_batch.clear();
        self.chunk_request_throttler.set_now(Instant::now());
        let player_chunk_position = world_to_chunk_pos(self.player.location.position);
        let render_threshold_sq = (self.config.render_distance as i32).pow(2);
        frustum_aabb.discrete_points(|chunk_position| {
            if player_chunk_position.distance_squared(chunk_position) > render_threshold_sq {
                return;
            }
            if let Some(chunk) = self.chunks.get(&chunk_position) {
                if !self.renderer.is_chunk_rendered(chunk_position) {
                    self.chunk_render_batch.push(chunk.clone());
                }
            } else if self.chunk_request_batch.len() < MAX_CHUNKS_PER_BATCH {
                let throttle_idx = smallhash::u32x3_to_18_bits(chunk_position.to_array());
                if self.chunk_request_throttler.request(throttle_idx as usize) {
                    self.chunk_request_batch.push(chunk_position);
                }
            }
        });
    }

    fn retain_frustum_chunks(&mut self, frustum_aabb: AABB) {
        self.renderer.retain_chunk_positions(|c_pos| {
            let min = c_pos.as_vec3();
            let chunk_aabb = AABB::new(min, min + 1.0);
            AABB::within_aabb(chunk_aabb, frustum_aabb)
        });
    }

    pub fn tick(&mut self) {
        let positions = std::mem::take(&mut self.unprocessed_chunk_positions);
        let mut updated = FxHashSet::default();
        for position in positions
            .into_iter()
            .map(|p| extended_with_preceding_positions(p))
            .flatten()
        {
            if updated.contains(&position) {
                continue;
            }
            updated.insert(position);
            self.update_chunk_data(position);
        }
    }

    fn update_chunk_data(&mut self, position: IVec3) {
        let adjacent_blocks = Array3D(compute::chunk::get_adjacent_blocks(position, &self.chunks));
        self.chunks.get_mut(&position).map(|chunk| {
            chunk.face_count = Some(compute::chunk::face_count(&chunk.blocks, &adjacent_blocks));
            chunk.adjacent_blocks = adjacent_blocks;
        });
    }
}

fn extended_with_preceding_positions(origin: IVec3) -> [IVec3; 4] {
    // fixme optimize redundant positions
    [
        origin,
        IVec3::new(origin.x - 1, origin.y, origin.z),
        IVec3::new(origin.x, origin.y - 1, origin.z),
        IVec3::new(origin.x, origin.y, origin.z - 1),
    ]
}
