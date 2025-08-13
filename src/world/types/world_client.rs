use crate::SIMULATION_AND_RENDER_DISTANCE;
use crate::app::app_renderer;
use crate::app::app_renderer::AppRenderer;
use crate::compute::ds::Slas;
use crate::world::types::Chunk;
use glam::IVec3;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use winit::window::Window;

pub struct WorldClient<'window> {
    pub renderer: AppRenderer<'window>,
    chunk_load_delta: HashMap<IVec3, Chunk>,
    chunk_unload_delta: HashSet<IVec3>,
    loaded_chunks: Slas<IVec3>,
}

impl<'window> WorldClient<'window> {
    pub fn new(window: Arc<Window>) -> Self {
        Self {
            renderer: app_renderer::make_app_renderer(
                window,
                SIMULATION_AND_RENDER_DISTANCE as f32,
            ),
            chunk_load_delta: HashMap::new(),
            chunk_unload_delta: HashSet::new(),
            loaded_chunks: Slas::new(),
        }
    }

    pub fn compare_for_delta(
        &self,
        server_chunks: &HashSet<IVec3>,
    ) -> (HashSet<IVec3>, HashSet<IVec3>) {
        let sym_diff = self.loaded_chunks.symmetric_difference(server_chunks);
        let mut to_unload_positions = HashSet::new();
        let mut to_load_positions = HashSet::new();
        for chunk_pos in sym_diff {
            if self.loaded_chunks.contains(chunk_pos) {
                to_unload_positions.insert(*chunk_pos);
            } else if !self.chunk_load_delta.contains_key(chunk_pos) {
                to_load_positions.insert(*chunk_pos);
            }
        }
        (to_load_positions, to_unload_positions)
    }

    pub fn update_chunks_by_delta(
        &mut self,
        new_chunks: Vec<(IVec3, Chunk)>,
        unload_positions: HashSet<IVec3>,
    ) {
        self.chunk_unload_delta.extend(unload_positions);
        self.chunk_load_delta.extend(new_chunks);
    }

    pub fn sync_with_renderer(&mut self) {
        let mut unload_delta = HashSet::new();
        std::mem::swap(&mut unload_delta, &mut self.chunk_unload_delta);
        self.loaded_chunks
            .retain(|chunk_pos| !unload_delta.contains(chunk_pos));
        self.renderer.unload_chunks(unload_delta);

        let mut load_delta = HashMap::new();
        std::mem::swap(&mut load_delta, &mut self.chunk_load_delta);
        let indexed_allocated_delta: Vec<_> = load_delta
            .into_iter()
            .map(|(c_pos, chunk)| (self.loaded_chunks.insert(c_pos), c_pos, chunk))
            .collect();
        self.renderer.write_new_chunks(indexed_allocated_delta);
        self.renderer.compute_chunks();
    }
}
