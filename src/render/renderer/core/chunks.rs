use crate::render::Renderer;
use crate::render::renderer;
use crate::render::renderer::resources::{ChunkPoolEntry, MeshBuffers};
use crate::render::types::Mesh;
use glam::IVec3;
use indexmap::IndexSet;
use std::collections::HashSet;

impl Renderer<'_> {
    pub(crate) fn add_chunk_buffer(&mut self, chunk_position: IVec3, chunk_mesh: &Mesh) {
        let (vert_alloc, ind_alloc) = renderer::alloc::size_of_mesh(chunk_mesh);
        let mesh_buffers = MeshBuffers {
            vertex: renderer::helpers::vertex::create_buffer(&self.device, vert_alloc),
            index: renderer::helpers::index::create_buffer(&self.device, ind_alloc),
        };
        self.queue.write_buffer(
            &mesh_buffers.vertex,
            0,
            bytemuck::cast_slice(&chunk_mesh.vertices),
        );
        self.queue.write_buffer(
            &mesh_buffers.index,
            0,
            bytemuck::cast_slice(&chunk_mesh.indices),
        );
        //
        // let chunk_entry = ChunkPoolEntry {
        //     mesh_buffers,
        //     index_offset: chunk_mesh.indices.len() as u32,
        // };
        // self.render_resources
        //     .chunk_pool
        //     .queue_load(chunk_position, chunk_entry);
    }

    pub(crate) fn remove_chunk_buffer(&mut self, index: usize) {
        // self.render_resources.chunk_pool.queue_remove(index);
        unimplemented!()
    }

    pub(crate) fn emerging_chunks(&mut self, active_positions: Vec<IVec3>) -> ! {
        // -> impl Iterator<Item = IVec3>
        // active_positions
        //     .into_iter()
        //     .filter(|c_pos| !self.render_resources.chunk_pool.contains(c_pos))
        unimplemented!()
    }

    pub(crate) fn expired_chunks(&mut self, active_positions: &HashSet<IVec3>) -> Vec<usize> {
        // returns asc sorted indices of unactive chunk entries
        unimplemented!()
        // let mut expired_chunks: Vec<usize> = self
        //     .render_resources
        //     .chunk_pool
        //     .iter()
        //     .enumerate()
        //     .filter(|(_, (c_pos, _))| !active_positions.contains(*c_pos))
        //     .map(|(c_idx, _)| c_idx)
        //     .collect();
        // expired_chunks.sort();
        // expired_chunks
    }
}
