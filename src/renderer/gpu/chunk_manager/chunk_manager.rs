use super::chunk_compute::ChunkCompute;
use super::chunk_render::ChunkRender;
use crate::call_every;
use crate::compute::ds::Slap;
use crate::renderer::Renderer;
use crate::renderer::gpu::{GPUVoxelChunk, GPUVoxelChunkAdjContent};
use crate::renderer::gpu::chunk_entry::GPUVoxelChunkHeader;
use crate::renderer::gpu::chunk_manager::BufferDrawArgs;
use crate::renderer::resources::vx_buffer::VxBuffer;
use crate::world::types::Chunk;
use glam::IVec3;
use std::collections::HashMap;
use suballoc::SubAllocator;

pub struct ChunkManager {
    suballocator: SubAllocator,
    suballocs: Slap<IVec3, u32>,
    active_draw: BufferDrawArgs,
    pub compute: ChunkCompute,
    pub render: ChunkRender,
}

impl ChunkManager {
    pub fn new(
        renderer: &Renderer<'_>,
        view_projection_buffer: &VxBuffer,
        max_face_count: usize,
        max_chunk_count: usize,
    ) -> Self {
        let render = ChunkRender::init(
            &renderer.device,
            view_projection_buffer,
            max_face_count,
        );
        let compute = ChunkCompute::init(&renderer.device, &render, max_chunk_count);
        Self {
            suballocator: SubAllocator::new(max_face_count as u32),
            suballocs: Slap::new(),
            active_draw: HashMap::new(),
            compute,
            render,
        }
    }

    pub fn is_rendered(&self, position: IVec3) -> bool {
        self.suballocs.contains(&position)
    }

    pub fn retain_chunk_positions<F>(&mut self, mut func: F)
    where
        F: FnMut(&IVec3) -> bool,
    {
        let to_drop = self
            .suballocs
            .iter()
            .filter_map(|(p, _)| func(p).then_some(p).cloned())
            .collect::<Vec<_>>();
        for p in to_drop {
            self.drop(p);
        }
    }

    pub fn write_new<'a>(
        &mut self,
        renderer: &Renderer<'_>,
        chunks: &mut impl Iterator<Item = &'a Chunk>,
    ) {
        let mut buffer_writes = Vec::new();
        for chunk in chunks {
            if self.suballocs.contains(&chunk.position) {
                // fixme this branch is not active
                // remeshing currently rendered chunk, drop first
                self.drop(chunk.position);
            }
            let face_count = chunk.face_count.unwrap() as u32;
            let mesh_alloc = self.suballocator.allocate(face_count).unwrap();
            let slab_index = self.suballocs.insert(chunk.position, mesh_alloc);

            let header = GPUVoxelChunkHeader::new(
                mesh_alloc as u32,
                face_count,
                slab_index as u32,
                chunk.position,
            );

            // fixme dereferencing from raw ptr could cause ub in the future
            let adjacent_blocks: GPUVoxelChunkAdjContent = unsafe {
                *(chunk.adjacent_blocks.as_ptr()
                    as *const GPUVoxelChunkAdjContent)
            };

            let entry = GPUVoxelChunk::new(header, adjacent_blocks, chunk.blocks);
            buffer_writes.push(entry);
        }
        self.compute.write_chunks(renderer, &buffer_writes);
        self.compute
            .dispatch_meshing_workgroups(renderer, &mut self.active_draw, buffer_writes);
    }

    pub fn draw(&mut self, renderer: &Renderer<'_>, render_pass: &mut wgpu::RenderPass) {
        // self.mem_debug_throttled();
        self.render
            .write_indirect_draw_args(renderer, &self.active_draw);
        self.render
            .draw(renderer, render_pass, self.active_draw.len() as u32);
    }

    pub fn drop(&mut self, position: IVec3) {
        let slap_entry_opt = self.suballocs.remove(&position);

        let (slab_index, alloc_start) = slap_entry_opt.unwrap();
        self.active_draw.remove(&slab_index).unwrap();
        self.suballocator.deallocate(alloc_start).unwrap();
    }

    pub fn mem_debug_throttled(&self) {
        // the blob clears cli
        let capacity = self.suballocator.capacity();
        let free_percent = (self.suballocator.free() as f32 / capacity as f32) * 100.0;
        call_every!(ALLOC_DBG, 50, || println!(
            "\x1B[2J\x1B[1;1Hfree: {:>3.1}% capacity: {}",
            free_percent, capacity,
        ));
    }
}
