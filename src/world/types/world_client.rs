use std::collections::HashSet;
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
    pub renderer: AppRenderer<'window, 2, 1>,
    nearby_chunks_delta: HashSet<IVec3>,
    nearby_chunks_neg_delta: HashSet<IVec3>,
    player_position: Vec3,
    nearby_chunks: Slas<IVec3>,
}

impl<'window> WorldClient<'window> {
    pub fn new(window: Arc<Window>, config: WorldClientConfig) -> Self {
        Self {
            renderer: app_renderer::make_app_renderer(window, config.render_distance as f32),
            config,
            nearby_chunks_delta: HashSet::new(),
            nearby_chunks_neg_delta: HashSet::new(),
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
                self.nearby_chunks_delta.insert(*chunk_position);
            }
        }
        for chunk_position in self.nearby_chunks.iter() {
            if !nearby_chunk_positions.contains(chunk_position) {
                self.nearby_chunks_neg_delta.insert(*chunk_position);
            }
        }
    }

    pub fn set_player_position(&mut self, position: Vec3) {
        self.player_position = position;
    }

    pub fn take_nearby_chunks_delta(&mut self) -> Vec<IVec3> {
        let mut new_delta = HashSet::new();
        std::mem::swap(&mut self.nearby_chunks_delta, &mut new_delta);
        new_delta.into_iter().collect()
    }

    fn take_nearby_chunks_neg_delta(&mut self) -> Vec<IVec3> {
        let mut neg_delta = HashSet::new();
        std::mem::swap(&mut self.nearby_chunks_neg_delta, &mut neg_delta);
        neg_delta.into_iter().collect()
    }

    pub fn sync_with_renderer(&mut self, new_chunks: Vec<(IVec3, Option<Chunk>)>) {
        let indexed_allocated_delta: Vec<_> = new_chunks
            .into_iter()
            .filter_map(|(c_pos, chunk_opt)| {
                chunk_opt.map(|chunk| {
                    if chunk.solid_count == 0 {
                        return None;
                    }
                    Some((self.nearby_chunks.insert(c_pos), chunk))
                }).unwrap_or(None)
            })
            .collect();
        if !indexed_allocated_delta.is_empty() {
            self.renderer.write_new_chunks(indexed_allocated_delta);
        }

        let neg_delta = self.take_nearby_chunks_neg_delta();
        for chunk_pos in neg_delta.iter() {
            self.nearby_chunks.remove(chunk_pos);
        }
        self.renderer.unload_chunks(neg_delta);
    }
}
