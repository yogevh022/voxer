use super::chunk_compute::ChunkCompute;
use super::chunk_render::ChunkRender;
use crate::compute::ds::Slap;
use crate::renderer::gpu::chunk_manager::{
    BufferDrawArgs, MeshAllocationRequest, MeshAllocator, StagingBufferMapping,
};
use crate::renderer::gpu::{VMallocMultiBuffer, VirtualMalloc};
use crate::renderer::{Index, Renderer, Vertex};
use crate::world::types::Chunk;
use glam::IVec3;
use std::array;
use std::collections::HashMap;

pub struct ChunkManager<const N_BUFF: usize, const N_STAGE_BUFF: usize> {
    mesh_allocator: MeshAllocator<N_BUFF>,
    chunk_allocations: Slap<IVec3, <MeshAllocator<N_BUFF> as VirtualMalloc>::Allocation>,
    active_draw: BufferDrawArgs<N_BUFF>,
    compute: ChunkCompute<N_STAGE_BUFF>,
    render: ChunkRender<N_BUFF>,
}

impl<const N_BUFF: usize, const N_STAGE_BUFF: usize> ChunkManager<N_BUFF, N_STAGE_BUFF> {
    pub fn new(
        renderer: &Renderer<'_>,
        vertex_buffer_size: usize,
        chunk_buffer_size: usize,
        mmat_buffer_size: usize,
    ) -> Self {
        let max_mesh_face_count = (vertex_buffer_size / size_of::<Vertex>()) / 4;
        let index_buffer_size = max_mesh_face_count * 6 * size_of::<Index>();
        let compute = ChunkCompute::<N_STAGE_BUFF>::init(
            &renderer.device,
            chunk_buffer_size as u64,
            vertex_buffer_size as u64,
            index_buffer_size as u64,
            mmat_buffer_size as u64,
        );
        let render = ChunkRender::<N_BUFF>::init(
            renderer,
            vertex_buffer_size as u64,
            index_buffer_size as u64,
            mmat_buffer_size as u64,
        );
        Self {
            mesh_allocator: VMallocMultiBuffer::new(max_mesh_face_count, 0),
            chunk_allocations: Slap::new(),
            active_draw: array::from_fn(|_| HashMap::new()),
            compute,
            render,
        }
    }

    pub fn is_rendered(&self, position: IVec3) -> bool {
        self.chunk_allocations.contains(&position)
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

    pub fn write_new<'a>(
        &mut self,
        renderer: &Renderer<'_>,
        chunks: &mut impl Iterator<Item = &'a Chunk>,
    ) {
        let max_faces_per_buffer = self.mesh_allocator.buffer_size();
        let staging_chunks_slices =
            chunks_slices_by_max_face_count(chunks, max_faces_per_buffer, N_STAGE_BUFF);
        for (staging_index, staging_slice) in staging_chunks_slices.into_iter().enumerate() {
            let mut staging_mapping = init_staging_mapping::<N_BUFF, N_STAGE_BUFF>();
            for (chunk, &face_count) in staging_slice.0.iter().zip(staging_slice.1.iter()) {
                if self.chunk_allocations.contains(&chunk.position) {
                    // remeshing currently rendered chunk, drop first
                    self.drop(chunk.position);
                }
                let target_buffer = self.buffer_index_for(chunk.position);
                let mesh_alloc = self
                    .mesh_allocator
                    .alloc(MeshAllocationRequest {
                        buffer_index: target_buffer,
                        size: face_count,
                    })
                    .unwrap();
                staging_mapping[staging_index].push_to(face_count as u64, mesh_alloc);
            }
            staging_mapping[staging_index].update_buffer_offsets();
            staging_mapping[staging_index].push_to_staging(&staging_slice.0, &mut self.chunk_allocations);
            self.compute.write_to_staging(renderer, &staging_mapping);
            self.compute.dispatch_staging_workgroups(
                renderer,
                &self.render,
                &mut self.active_draw,
                staging_mapping,
            );
        }
    }

    fn buffer_index_for(&self, position: IVec3) -> usize {
        position.element_sum() as usize % N_BUFF
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
        self.active_draw[full_alloc.buffer_index].remove(&slab_index);
        if let Err(e) = self.mesh_allocator.free(full_alloc) {
            // fixme this should never happen
            // println!("malloc::free failed for {:?}, {:?}", position, e);
        }
    }

    pub fn malloc_debug(&self) {
        println!("\x1B[2J\x1B[1;1H{}", self.mesh_allocator); // the blob clears cli
    }
}

const fn init_staging_mapping<const NUM_BUFFERS: usize, const NUM_STAGING_BUFFERS: usize>()
-> [StagingBufferMapping<NUM_BUFFERS>; NUM_STAGING_BUFFERS] {
    [const { StagingBufferMapping::<NUM_BUFFERS>::new() }; NUM_STAGING_BUFFERS]
}

fn chunks_slices_by_max_face_count<'a>(
    chunks: &mut impl Iterator<Item = &'a Chunk>,
    max_faces_per_buffer: usize,
    max_slices: usize,
) -> Vec<(Vec<&'a Chunk>, Vec<usize>)> {
    let mut slices = Vec::new();
    let mut this_slice = Vec::new();
    let mut face_counts = Vec::new();
    let mut acc = 0usize;
    if let Some(chunk) = chunks.next() {
        let face_count = chunk.face_count.unwrap();
        if acc + face_count < max_faces_per_buffer {
            acc += face_count;
            this_slice.push(chunk);
            face_counts.push(face_count);
        } else {
            slices.push((
                std::mem::take(&mut this_slice),
                std::mem::take(&mut face_counts),
            ));
            face_counts.push(face_count);
            if slices.len() == max_slices {
                return slices;
            }
        }
    }
    slices.push((this_slice, face_counts));
    slices
}
