use crate::app::app_renderer;
use crate::app::app_renderer::AppRenderer;
use crate::avg;
use crate::compute::ds::Slas;
use crate::compute::geo;
use crate::world::types::{CHUNK_DIM, Chunk};
use glam::{IVec3, Vec3};
use std::io::Write;
use std::sync::Arc;
use winit::window::Window;

pub struct WorldClientConfig {
    pub render_distance: usize,
}

pub struct WorldClient<'window> {
    pub config: WorldClientConfig,
    pub renderer: AppRenderer<'window, 4>,
    nearby_chunks_delta: Vec<IVec3>,
    nearby_chunks_neg_delta: Vec<IVec3>,
    player_position: Vec3,
    nearby_chunks: Slas<IVec3>,
}

impl<'window> WorldClient<'window> {
    pub fn new(window: Arc<Window>, config: WorldClientConfig) -> Self {
        Self {
            renderer: app_renderer::make_app_renderer(window, config.render_distance as f32),
            config,
            nearby_chunks_delta: Vec::new(),
            nearby_chunks_neg_delta: Vec::new(),
            player_position: Vec3::ZERO,
            nearby_chunks: Slas::new(),
        }
    }

    pub fn update(&mut self) {
        // fixme slow code
        let nearby_chunk_positions = geo::discrete_sphere_pts(
            &(self.player_position / CHUNK_DIM as f32),
            self.config.render_distance as f32,
        );
        for chunk_position in nearby_chunk_positions.iter() {
            if !self.nearby_chunks.contains(&chunk_position) {
                self.nearby_chunks_delta.push(*chunk_position);
            }
        }
        for chunk_position in self.nearby_chunks.iter() {
            if !nearby_chunk_positions.contains(chunk_position) {
                self.nearby_chunks_neg_delta.push(*chunk_position);
            }
        }
    }

    pub fn set_player_position(&mut self, position: Vec3) {
        self.player_position = position;
    }

    pub fn take_nearby_chunks_delta(&mut self) -> Vec<IVec3> {
        let mut new_delta = Vec::new();
        std::mem::swap(&mut self.nearby_chunks_delta, &mut new_delta);
        new_delta
    }

    fn take_nearby_chunks_neg_delta(&mut self) -> Vec<IVec3> {
        let mut neg_delta = Vec::new();
        std::mem::swap(&mut self.nearby_chunks_neg_delta, &mut neg_delta);
        neg_delta
    }

    pub fn sync_with_renderer(&mut self, new_chunks: Vec<(IVec3, Option<Chunk>)>) {
        let neg_delta = self.take_nearby_chunks_neg_delta();
        for chunk_pos in neg_delta.iter() {
            self.nearby_chunks.remove(chunk_pos);
        }
        self.renderer.unload_chunks(neg_delta);

        let indexed_allocated_delta: Vec<_> = new_chunks
            .into_iter()
            .filter_map(|(c_pos, chunk_opt)| {
                chunk_opt.map(|chunk| (self.nearby_chunks.insert(c_pos), c_pos, chunk))
            })
            .collect();
        if !indexed_allocated_delta.is_empty() {
            self.renderer.write_new_chunks(indexed_allocated_delta);
            self.renderer.compute_chunks();
        }
    }
}
