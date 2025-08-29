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
        let max_mesh_face_count = (vertex_buffer_size / size_of::<Vertex>()) / 4;
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
        let mut chunk_i = 0;
        let mut face_count_acc = 0;
        let mut target_staging_buffer = 0;
        let max_faces_per_buffer = 0;
        let mut staging_entries = init_staging_entries::<NumStagingBuffers>();
        let mut staging_mapping = init_staging_mapping::<NumBuffers, NumStagingBuffers>();
        loop {
            let chunk = &chunks[chunk_i];
            let face_count = compute::chunk::face_count(&chunk.blocks);
            if face_count_acc + face_count > max_faces_per_buffer || chunk_i == chunks.len() - 1 {
                staging_mapping
                    .iter_mut()
                    .for_each(|mapping| mapping.update_buffer_offsets());
                staging_mapping[target_staging_buffer].push_to_staging(
                    &chunks,
                    &mut staging_entries[target_staging_buffer],
                    |chunk_position, mesh_allocation| {
                        self.chunk_allocations
                            .insert(chunk_position, mesh_allocation)
                    },
                );

                self.compute
                    .write_to_staging_chunks(renderer, &staging_entries);
                let mut s_entries = init_staging_entries::<NumStagingBuffers>();
                let mut s_mapping = init_staging_mapping::<NumBuffers, NumStagingBuffers>();
                std::mem::swap(&mut s_entries, &mut staging_entries);
                std::mem::swap(&mut s_mapping, &mut staging_mapping);
                self.compute.dispatch_staging_workgroups(
                    renderer,
                    &self.render,
                    s_entries,
                    s_mapping,
                    &self.delta_draw,
                );

                if target_staging_buffer == NumStagingBuffers - 1 {
                    return;
                }
                target_staging_buffer += 1;
            }
            debug_assert_eq!(self.chunk_allocations.get(&chunk.position).is_some(), false);
            let target_buffer = self.buffer_index_for(chunk.position);

            let mesh_alloc = self
                .mesh_allocator
                .alloc(MeshAllocationRequest {
                    buffer_index: target_buffer,
                    size: face_count,
                })
                .unwrap();

            staging_mapping[target_staging_buffer].push_to(face_count as u64, mesh_alloc);
            face_count_acc += face_count;
            chunk_i += 1;
        }
    }

    fn buffer_index_for(&self, position: IVec3) -> usize {
        position.element_sum() as usize % NumBuffers
    }

    fn staging_index_for(&self, chunk_slice_i: usize) -> usize {
        chunk_slice_i % NumStagingBuffers
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
        let buffer_index = self.buffer_index_for(position);
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
        println!("\x1B[2J\x1B[1;1H{}", self.mesh_allocator); // the blob clears cli
    }
}

const fn init_staging_entries<const NUM_STAGING_BUFFERS: usize>()
-> [Vec<GPUChunkEntry>; NUM_STAGING_BUFFERS] {
    [const { Vec::<GPUChunkEntry>::new() }; NUM_STAGING_BUFFERS]
}

const fn init_staging_mapping<const NUM_BUFFERS: usize, const NUM_STAGING_BUFFERS: usize>()
-> [StagingBufferMapping<NUM_BUFFERS>; NUM_STAGING_BUFFERS] {
    [const { StagingBufferMapping::<NUM_BUFFERS>::new() }; NUM_STAGING_BUFFERS]
}

fn chunks_slices_by_max_face_count(
    chunks: &Vec<Chunk>,
    max_faces_per_chunk: usize,
) -> Vec<(&[Chunk], Vec<usize>)> {
    let mut slices = Vec::new();
    let mut face_counts = Vec::new();
    let mut acc = 0usize;
    let mut i = 0usize;
    let mut j = 0usize;
    while j < chunks.len() {
        let face_count = compute::chunk::face_count(&chunks[j].blocks);
        if acc + face_count < max_faces_per_chunk {
            acc += face_count;
            j += 1;
            face_counts.push(face_count);
        } else {
            slices.push((&chunks[i..j], std::mem::take(&mut face_counts)));
            acc = face_count;
            i = j;
            j += 1;
        }
    }
    slices
}
