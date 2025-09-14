use super::chunk_compute::ChunkCompute;
use super::chunk_render::ChunkRender;
use crate::compute::ds::Slap;
use crate::renderer::gpu::chunk_entry::GPUChunkEntryHeader;
use crate::renderer::gpu::chunk_manager::{BufferDrawArgs, MeshAllocationRequest};
use crate::renderer::gpu::{GPUChunkEntry, VMallocFirstFit, VMallocMultiBuffer, VirtualMalloc};
use crate::renderer::{Index, Renderer, Vertex};
use crate::world::types::{CHUNK_DIM, Chunk, PACKED_CHUNK_DIM};
use glam::IVec3;
use std::array;
use std::collections::HashMap;

pub struct ChunkManager {
    mesh_allocator: VMallocFirstFit,
    chunk_allocations: Slap<IVec3, <VMallocFirstFit as VirtualMalloc>::Allocation>,
    active_draw: BufferDrawArgs,
    pub compute: ChunkCompute,
    pub render: ChunkRender,
}

impl ChunkManager {
    pub fn new(
        renderer: &Renderer<'_>,
        face_data_buffer_size: wgpu::BufferAddress,
        chunks_buffer_size: wgpu::BufferAddress,
        chunk_translations_buffer_size: wgpu::BufferAddress,
        fd_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let render = ChunkRender::init(
            renderer,
            face_data_buffer_size,
            chunk_translations_buffer_size,
            fd_layout
        );
        let compute = ChunkCompute::init(&renderer.device, &render, chunks_buffer_size);
        Self {
            mesh_allocator: VMallocFirstFit::new(face_data_buffer_size as usize, 0),
            chunk_allocations: Slap::new(),
            active_draw: HashMap::new(),
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
        let mut buffer_writes = Vec::new();
        for chunk in chunks {
            if self.chunk_allocations.contains(&chunk.position) {
                // remeshing currently rendered chunk, drop first
                self.drop(chunk.position);
            }
            let face_count = chunk.face_count.unwrap();
            let mesh_alloc = self
                .mesh_allocator
                .alloc(MeshAllocationRequest { size: face_count })
                .unwrap();
            let slab_index = self.chunk_allocations.insert(chunk.position, mesh_alloc);

            let header = GPUChunkEntryHeader::new(
                mesh_alloc.offset as u32,
                face_count as u32,
                slab_index as u32,
                chunk.position,
            );

            // fixme dereferencing from raw ptr could cause ub in the future
            let adjacent_blocks: [[[u32; PACKED_CHUNK_DIM]; CHUNK_DIM]; 3] = unsafe {
                *(chunk.adjacent_blocks.as_ptr()
                    as *const [[[u32; PACKED_CHUNK_DIM]; CHUNK_DIM]; 3])
            };

            let entry = GPUChunkEntry::new(header, adjacent_blocks, chunk.blocks);
            buffer_writes.push(entry);
        }
        self.compute.write_chunks(renderer, &buffer_writes);
        self.compute
            .dispatch_staging_workgroups(renderer, &mut self.active_draw, buffer_writes);
    }

    pub fn draw(&mut self, renderer: &Renderer<'_>, render_pass: &mut wgpu::RenderPass) {
        self.render
            .write_args_to_indirect_buffer(renderer, &self.active_draw);
        self.render.draw(renderer, render_pass, self.active_draw.len() as u32);
    }

    pub fn drop(&mut self, position: IVec3) {
        let slap_entry_opt = self.chunk_allocations.remove(&position);
        let (slab_index, full_alloc) = slap_entry_opt.unwrap();
        self.active_draw.remove(&slab_index);
        if let Err(e) = self.mesh_allocator.free(full_alloc) {
            // fixme this should never happen
            // println!("malloc::free failed for {:?}, {:?}", position, e);
        }
    }

    pub fn malloc_debug(&self) {
        println!("\x1B[2J\x1B[1;1H{:?}", self.mesh_allocator); // the blob clears cli
    }
}
