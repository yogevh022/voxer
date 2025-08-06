use crate::render::renderer::alloc;
use crate::render::renderer::resources::MeshBuffers;
use crate::render::types::Mesh;
use crate::render::{Renderer, index, vertex};
use glam::IVec3;

pub(crate) struct ChunkBufferEntry {
    pub buffer: MeshBuffers,
    pub position: IVec3,
    pub index_offset: u32,
}

impl Renderer<'_> {
    pub(crate) fn add_chunk_buffer(&mut self, chunk_position: IVec3, chunk_mesh: &Mesh) {
        let (vert_alloc, ind_alloc) = alloc::size_of_mesh(chunk_mesh);
        let mesh_buffers = MeshBuffers {
            vertex: vertex::create_buffer(&self.device, vert_alloc),
            index: index::create_buffer(&self.device, ind_alloc),
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

        let chunk_buffer_entry = ChunkBufferEntry {
            buffer: mesh_buffers,
            position: chunk_position,
            index_offset: chunk_mesh.indices.len() as u32,
        };
        self.resources.chunk_buffer_pool.push(chunk_buffer_entry);
    }

    pub(crate) fn remove_chunk_buffer(&mut self, index: usize) {
        self.resources.chunk_buffer_pool.remove(index);
    }

    pub(crate) fn emerging_chunks(
        &mut self,
        active_positions: Vec<IVec3>,
    ) -> Vec<IVec3> {
        active_positions
            .into_iter()
            .filter(|c_pos| {
                self.resources
                    .chunk_buffer_pool
                    .iter()
                    .position(|entry| entry.position == *c_pos)
                    .is_none()
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn expired_chunks(&mut self, active_positions: &[IVec3]) -> Vec<usize> {
        // returns asc sorted indices of unactive chunk entries
        // todo optimize (better ds than vec)
        let mut expired_chunks: Vec<usize> = self
            .resources
            .chunk_buffer_pool
            .iter()
            .enumerate()
            .filter(|(_, buff_entry)| !active_positions.contains(&buff_entry.position))
            .map(|(c_idx, _)| c_idx)
            .collect();
        expired_chunks.sort();
        expired_chunks
    }
}
