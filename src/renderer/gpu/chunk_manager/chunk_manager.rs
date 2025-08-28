use super::chunk_compute::ChunkCompute;
use super::chunk_render::ChunkRender;
use crate::compute;
use crate::compute::ds::Slap;
use crate::renderer::gpu::chunk_entry::GPUChunkEntryHeader;
use crate::renderer::gpu::chunk_manager::{
    BufferDrawArgs, MeshAllocationRequest, MeshAllocator, StagingBufferMapping,
};
use crate::renderer::gpu::{GPUChunkEntry, VMallocMultiBuffer, VirtualMalloc};
use crate::renderer::{Index, Renderer, Vertex};
use crate::world::types::Chunk;
use glam::IVec3;
use parking_lot::Mutex;
use std::array;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ChunkManager<const NumBuffers: usize, const NumStagingBuffers: usize> {
    mesh_allocator: MeshAllocator<NumBuffers>,
    chunk_allocations: Slap<IVec3, <MeshAllocator<NumBuffers> as VirtualMalloc>::Allocation>,
    active_draw: BufferDrawArgs<NumBuffers>,
    delta_draw: Arc<Mutex<Option<BufferDrawArgs<NumBuffers>>>>,
    compute: ChunkCompute<NumStagingBuffers>,
    render: ChunkRender<NumBuffers>,
}

impl<const NumBuffers: usize, const NumStagingBuffers: usize>
    ChunkManager<NumBuffers, NumStagingBuffers>
{
    pub fn new(
        renderer: &Renderer<'_>,
        vertex_buffer_size: usize,
        chunk_buffer_size: usize,
        mmat_buffer_size: usize,
    ) -> Self {
        let max_mesh_face_count = (vertex_buffer_size / Vertex::size()) / 4;
        let index_buffer_size = max_mesh_face_count * 6 * size_of::<Index>();
        let compute = ChunkCompute::<NumStagingBuffers>::init(
            &renderer.device,
            chunk_buffer_size as u64,
            vertex_buffer_size as u64,
            index_buffer_size as u64,
            mmat_buffer_size as u64,
        );
        let render = ChunkRender::<NumBuffers>::init(
            renderer,
            vertex_buffer_size as u64,
            index_buffer_size as u64,
            mmat_buffer_size as u64,
        );
        Self {
            mesh_allocator: VMallocMultiBuffer::new(max_mesh_face_count, 0),
            chunk_allocations: Slap::new(),
            active_draw: array::from_fn(|_| HashMap::new()),
            delta_draw: Arc::new(Mutex::new(None)),
            compute,
            render,
        }
    }

    pub fn is_rendered(&self, position: IVec3) -> bool {
        self.chunk_allocations.get(&position).is_some()
    }

    pub fn map_rendered_chunk_positions<F>(&self, mut func: F) -> Vec<IVec3>
    where
        F: FnMut(IVec3) -> bool,
    {
        self.chunk_allocations
            .iter()
            .filter_map(|(&chunk_position, _)| func(chunk_position).then_some(chunk_position))
            .collect()
    }

    pub fn write_new(&mut self, renderer: &Renderer<'_>, chunks: Vec<Chunk>) {
        let mut staging_entries = [const { Vec::<GPUChunkEntry>::new() }; NumStagingBuffers];
        let mut staging_mapping =
            [const { StagingBufferMapping::<NumBuffers>::new() }; NumStagingBuffers];
        for chunk in chunks.iter() {
            debug_assert_eq!(self.chunk_allocations.get(&chunk.position).is_some(), false);
            let target_buffer = self.target_buffer_for(chunk.position);
            let target_staging_buffer = self.target_staging_buffer_for(chunk.position);
            let face_count = compute::chunk::face_count(&chunk.blocks);

            let mesh_alloc = self
                .mesh_allocator
                .alloc(MeshAllocationRequest {
                    buffer_index: target_buffer,
                    size: face_count,
                })
                .unwrap();

            staging_mapping[target_staging_buffer].push_to(face_count, mesh_alloc);
        }
        staging_mapping
            .iter_mut()
            .for_each(|mapping| mapping.update_buffer_offsets());
        for (i, chunk) in chunks.into_iter().enumerate() {
            // fixme DRY
            let target_staging_buffer = self.target_staging_buffer_for(chunk.position);
            let mapping = &staging_mapping[target_staging_buffer];

            let mesh_alloc = mapping.targets[i].allocation;
            let face_count = mapping.targets[i].size;
            let slab_index = self.chunk_allocations.insert(chunk.position, mesh_alloc);
            let staging_offset =
                mapping.buffer_offsets[mesh_alloc.buffer_index] + mapping.targets[i].entry_offset;

            let header = GPUChunkEntryHeader::new(
                staging_offset as u32,
                mesh_alloc.offset as i32 - staging_offset as i32,
                face_count as u32,
                slab_index as u32,
                chunk.position,
            );
            staging_entries[target_staging_buffer].push(GPUChunkEntry::new(header, chunk.blocks));
        }

        self.compute
            .write_to_staging_chunks(renderer, &staging_entries);
        self.compute.dispatch_staging_workgroups(
            renderer,
            &self.render,
            staging_entries,
            staging_mapping,
            &self.delta_draw,
        );
    }

    fn target_buffer_for(&self, position: IVec3) -> usize {
        position.element_sum() as usize % NumBuffers
    }

    fn target_staging_buffer_for(&self, _position: IVec3) -> usize {
        0usize
    }

    pub fn draw(&mut self, renderer: &Renderer<'_>, render_pass: &mut wgpu::RenderPass) {
        let draw_instructions = self
            .render
            .write_args_to_indirect_buffer(renderer, &self.active_draw);
        self.render
            .multi_draw(renderer, render_pass, draw_instructions);
    }

    pub fn drop(&mut self, position: IVec3) {
        let slap_entry_opt = self.chunk_allocations.remove(&position);
        let (slab_index, full_alloc) = slap_entry_opt.unwrap();
        let buffer_index = self.target_buffer_for(position);
        self.active_draw[buffer_index].remove(&slab_index);
        if let Err(e) = self.mesh_allocator.free(full_alloc) {
            // fixme this should never happen
            // println!("malloc::free failed for {:?}, {:?}", position, e);
        }
    }

    pub fn poll_update_delta_draw(&mut self) {
        let delta = {
            let mut guard = self.delta_draw.lock();
            guard.take()
        };
        if let Some(delta) = delta {
            for (i, args) in delta.into_iter().enumerate() {
                self.active_draw[i].extend(args);
            }
        }
    }

    pub fn malloc_debug(&self) {
        todo!()
        // self.mesh_allocator.debug();
    }
}
