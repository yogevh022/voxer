use crate::render::encoders;
use crate::render::renderer::core::Renderer;
use crate::render::renderer::gpu;
use crate::render::renderer::resources::ChunkPool;
use crate::world::types::World;
use crate::{types, utils};
use glam::Mat4;
use std::mem;
use wgpu::CommandEncoder;

impl Renderer<'_> {
    pub(crate) fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        todo!();
        // FIRST REMOVE PENDING REMOVALS
        // let chunk_pool_indices = self.render_resources.chunk_pool.take_remove_queue();
        // if !chunk_pool_indices.is_empty() {
        //     // minimal traffic swap remove algorithm for the buffer
        //     let new_indices = ChunkPool::post_swap_remove_indices(
        //         self.render_resources.chunk_pool.len(),
        //         &chunk_pool_indices,
        //     );
        //     gpu::buffer::reorder_to_indices(
        //         &self.queue,
        //         &self.render_resources.transform.model_matrix_buffer,
        //         size_of::<[[f32; 4]; 4]>(),
        //         &new_indices,
        //         |i| {
        //             utils::mat::model_matrix(self.render_resources.chunk_pool.get_index(i).0)
        //                 .to_cols_array()
        //         },
        //     );
        //
        //     // plain swap remove on cpu
        //     for i in chunk_pool_indices {
        //         self.render_resources.chunk_pool.swap_remove(i);
        //     }
        // }

        // THEN PUSH PENDING ADDITIONS
        // let chunk_entries = self.render_resources.chunk_pool.take_load_queue();
        // if !chunk_entries.is_empty() {
        //     let initial_count = self.render_resources.chunk_pool.len();
        //     self.queue.write_buffer(
        //         &self.render_resources.transform.model_matrix_buffer,
        //         (initial_count * size_of::<[[f32; 4]; 4]>()) as u64,
        //         bytemuck::cast_slice(
        //             &chunk_entries
        //                 .iter()
        //                 .map(|(c_pos, _)| utils::mat::model_matrix(c_pos))
        //                 .collect::<Vec<_>>(),
        //         ),
        //     );
        //     self.render_resources.chunk_pool.extend(chunk_entries);
        // }

        // encode main render pass
        // encoders::render_pass::encode(
        //     &mut encoder,
        //     encoders::render_pass::RenderPassEncodeContext {
        //         view,
        //         pipeline: &self.render_pipeline,
        //         device: &self.device,
        //         queue: &self.queue,
        //         resources: &self.render_resources,
        //         view_projection: camera.get_view_projection(),
        //     },
        // );
        
    }
}
