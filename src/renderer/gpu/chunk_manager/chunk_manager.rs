use super::chunk_compute::ChunkCompute;
use super::chunk_render::ChunkRender;
use crate::compute;
use crate::renderer::gpu::chunk_manager::BufferDrawArgs;
use crate::renderer::gpu::{
    GPUChunkEntry, GPUChunkEntryHeader, MeshVMallocMultiBuffer, MultiBufferMeshAllocationRequest,
};
use crate::renderer::{Index, Renderer, Vertex};
use crate::world::types::Chunk;
use parking_lot::RwLock;
use std::array;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ChunkManager<const NumBuffers: usize, const NumStagingBuffers: usize> {
    mesh_allocator: MeshVMallocMultiBuffer<NumBuffers>,
    current_draw: BufferDrawArgs<NumBuffers>,
    delta_draw: Arc<RwLock<Option<BufferDrawArgs<NumBuffers>>>>,
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
            current_draw: array::from_fn(|_| HashMap::new()),
            delta_draw: Arc::new(RwLock::new(None)),
            compute,
            render,
        }
    }

    pub fn write_new(&mut self, renderer: &Renderer<'_>, chunks: Vec<(usize, Chunk)>) {
        let mut staging_write = [const { Vec::<GPUChunkEntry>::new() }; NumStagingBuffers];
        let mut staging_targets = [const { Vec::<usize>::new() }; NumStagingBuffers];
        for (slab_index, chunk) in chunks.into_iter() {
            let face_count = compute::chunk::face_count(&chunk.blocks);
            let vertex_count = face_count * 4;
            let index_count = face_count * 6;
            let target_buffer = chunk.position.element_sum() as usize % NumBuffers;
            let target_staging_buffer = 0usize;
            let alloc_request = MultiBufferMeshAllocationRequest {
                buffer_index: target_buffer,
                vertex_size: vertex_count,
                index_size: index_count,
            };
            let mesh_alloc = self.mesh_allocator.alloc(alloc_request).unwrap();
            let header = GPUChunkEntryHeader::new(mesh_alloc, slab_index as u32, chunk.position);
            staging_write[target_staging_buffer].push(GPUChunkEntry::new(header, chunk.blocks));
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

    pub fn draw(&mut self, renderer: &Renderer<'_>, render_pass: &mut wgpu::RenderPass) {
        let draw_instructions = self
            .render
            .write_args_to_indirect_buffer(renderer, &self.current_draw);
        self.render.multi_draw(renderer, render_pass, draw_instructions);
    }

    pub fn poll_update_delta_draw(&mut self) {
        let delta = {
            let mut guard = self.delta_draw.write();
            guard.take()
        };
        delta.map(|buffer_draw_args| {
            buffer_draw_args.into_iter().enumerate().map(|(i, args)| {
                self.current_draw[i].extend(args);
            })
        });
    }
}
