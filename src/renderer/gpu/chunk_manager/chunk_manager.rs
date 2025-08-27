use super::chunk_compute::ChunkCompute;
use super::chunk_render::ChunkRender;
use crate::compute;
use crate::compute::ds::Slap;
use crate::renderer::gpu::chunk_manager::BufferDrawArgs;
use crate::renderer::gpu::{
    GPUChunkEntry, MeshVMallocMultiBuffer, MultiBufferMeshAllocation,
    MultiBufferMeshAllocationRequest,
};
use crate::renderer::{Index, Renderer, Vertex};
use crate::world::types::Chunk;
use glam::IVec3;
use parking_lot::Mutex;
use std::array;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct ChunkManager<const NumBuffers: usize, const NumStagingBuffers: usize> {
    mesh_allocator: MeshVMallocMultiBuffer<NumBuffers>,
    chunks_slap: Slap<IVec3, (usize, MultiBufferMeshAllocation)>,
    current_draw: BufferDrawArgs<NumBuffers>,
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
        let vertex_index_ratio = size_of::<Vertex>() / size_of::<Index>();
        let index_buffer_size =
            ((vertex_buffer_size as f32 / vertex_index_ratio as f32) * 1.5) as usize;
        let vertices_per_buffer = vertex_buffer_size / size_of::<Vertex>();
        let indices_per_buffer = index_buffer_size / size_of::<Index>();
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
            mesh_allocator: MeshVMallocMultiBuffer::new(vertices_per_buffer, indices_per_buffer, 0),
            chunks_slap: Slap::new(),
            current_draw: array::from_fn(|_| HashMap::new()),
            delta_draw: Arc::new(Mutex::new(None)),
            compute,
            render,
        }
    }

    pub fn is_rendered(&self, position: IVec3) -> bool {
        self.chunks_slap.get(&position).is_some()
    }

    pub fn map_rendered_chunk_positions<F>(&self, mut func: F) -> Vec<IVec3>
    where
        F: FnMut(IVec3) -> bool,
    {
        self.chunks_slap
            .iter()
            .filter_map(|(&chunk_position, _)| func(chunk_position).then_some(chunk_position))
            .collect()
    }

    pub fn write_new(&mut self, renderer: &Renderer<'_>, chunks: Vec<Chunk>) {
        let mut staging_write = [const { Vec::<GPUChunkEntry>::new() }; NumStagingBuffers];
        let mut staging_targets = [const { Vec::<usize>::new() }; NumStagingBuffers];
        for chunk in chunks.into_iter() {
            debug_assert_eq!(self.chunks_slap.get(&chunk.position).is_some(), false);
            let target_buffer = self.target_buffer_for(chunk.position);
            let target_staging_buffer = self.target_staging_buffer_for(chunk.position);

            let face_count = compute::chunk::face_count(&chunk.blocks);
            let alloc_request = MultiBufferMeshAllocationRequest {
                buffer_index: target_buffer,
                vertex_count: face_count * 4,
                index_count: face_count * 6,
            };
            let mesh_alloc = self.mesh_allocator.alloc(alloc_request).unwrap();
            let slab_index = self
                .chunks_slap
                .insert(chunk.position, (target_buffer, mesh_alloc));

            staging_write[target_staging_buffer].push(GPUChunkEntry::new(
                mesh_alloc,
                slab_index as u32,
                chunk.position,
                chunk.blocks,
            ));
            staging_targets[target_staging_buffer].push(target_buffer);
        }
        self.compute
            .write_to_staging_chunks(renderer, &staging_write);
        self.compute.dispatch_staging_workgroups(
            renderer,
            &self.render,
            staging_write,
            staging_targets,
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
            .write_args_to_indirect_buffer(renderer, &self.current_draw);
        self.render
            .multi_draw(renderer, render_pass, draw_instructions);
    }

    pub fn drop(&mut self, position: IVec3) {
        let slap_entry_opt = self.chunks_slap.remove(&position);
        let (slab_index, full_alloc) = slap_entry_opt.unwrap();
        let buffer_index = self.target_buffer_for(position);
        self.current_draw[buffer_index].remove(&slab_index);
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
                self.current_draw[i].extend(args);
            }
        }
    }

    pub fn malloc_debug(&self) {
        self.mesh_allocator.debug();
    }
}
