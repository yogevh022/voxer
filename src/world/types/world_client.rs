use crate::app::app_renderer;
use crate::app::app_renderer::AppRenderer;
use crate::compute;
use crate::compute::geo;
use crate::world::types::Chunk;
use glam::{IVec3, Vec3};
use std::sync::Arc;
use winit::window::Window;

pub struct WorldClientConfig {
    pub render_distance: usize,
}

pub struct WorldClient<'window> {
    pub config: WorldClientConfig,
    pub renderer: AppRenderer<'window, 1, 1>,
    last_chunk_position: IVec3,
}

impl<'window> WorldClient<'window> {
    pub fn new(window: Arc<Window>, config: WorldClientConfig) -> Self {
        Self {
            renderer: app_renderer::make_app_renderer(window, config.render_distance as f32),
            config,
            last_chunk_position: IVec3::ZERO,
        }
    }

    pub fn set_chunk_position(&mut self, position: IVec3) {
        self.last_chunk_position = position;
    }

    pub fn nearby_chunks_delta(&mut self, new_chunk_position: IVec3) -> (Vec<IVec3>, Vec<IVec3>) {
        let mut nearby_delta = Vec::new();
        geo::Sphere::point_delta(
            new_chunk_position,
            self.last_chunk_position,
            self.config.render_distance as isize,
            |point| {
                nearby_delta.push(point);
            },
        );

        let mut nearby_neg_delta = Vec::new();
        geo::Sphere::point_delta(
            self.last_chunk_position,
            new_chunk_position,
            self.config.render_distance as isize,
            |point| {
                nearby_neg_delta.push(point);
            },
        );

        (nearby_delta, nearby_neg_delta)
    }

    pub fn send_chunks_to_renderer(&mut self, new_chunks: Vec<Option<Chunk>>) {
        let new_chunks_filtered = new_chunks
            .into_iter()
            .filter_map(|chunk_opt| {
                chunk_opt
                    .map(|c| (c.solid_count > 0).then_some(c))
                    .unwrap_or(None)
            })
            .collect::<Vec<_>>();

        if !new_chunks_filtered.is_empty() {
            self.renderer.write_new_chunks(new_chunks_filtered);
        }
    }

    // pub fn sync_with_renderer(
    //     &mut self,
    //     new_chunks: Vec<Option<Chunk>>,
    //     outdated_chunks: Vec<IVec3>,
    // ) {
    //     let indexed_allocated_delta: Vec<_> = new_chunks
    //         .into_iter()
    //         .filter_map(|chunk_opt| {
    //             chunk_opt
    //                 .map(|chunk| {
    //                     if chunk.solid_count == 0 {
    //                         return None;
    //                     }
    //                     Some((self.nearby_chunks.insert(c_pos), chunk))
    //                 })
    //                 .unwrap_or(None)
    //         })
    //         .collect();
    //     if !indexed_allocated_delta.is_empty() {
    //         self.renderer.write_new_chunks(indexed_allocated_delta);
    //     }
    //
    //     for chunk_pos in outdated_chunks.iter() {
    //         self.nearby_chunks.remove(chunk_pos);
    //     }
    //     self.renderer.unload_chunks(outdated_chunks);
    // }
}
