mod chunk_compute;
mod chunk_manager;
mod chunk_render;

use crate::compute::ds::Slap;
use crate::renderer::DrawIndexedIndirectArgsA32;
use crate::renderer::gpu::chunk_entry::GPUChunkEntryHeader;
use crate::renderer::gpu::{
    GPUChunkEntry, VMallocFirstFit, VMallocMultiBuffer, VirtualMalloc,
};
use crate::world::types::{CHUNK_DIM, PACKED_CHUNK_DIM, Chunk};
pub use chunk_manager::ChunkManager;
use glam::IVec3;
use std::collections::HashMap;

type BufferDrawArgs<const N: usize> = [HashMap<usize, DrawIndexedIndirectArgsA32>; N];
type MeshAllocator<const N: usize> = VMallocMultiBuffer<VMallocFirstFit, N>;
type MeshAllocationRequest =
    <VMallocMultiBuffer<VMallocFirstFit, 0> as VirtualMalloc>::AllocationRequest;
type MeshAllocation = <VMallocMultiBuffer<VMallocFirstFit, 0> as VirtualMalloc>::Allocation;

#[derive(Debug)]
struct MultiDrawInstruction {
    offset: usize,
    count: usize,
}

#[derive(Debug, Clone)]
struct BufferCopyTarget {
    pub entry_offset: u64,
    pub size: u64,
    pub allocation: MeshAllocation,
}

#[derive(Debug, Clone)]
struct StagingBufferMapping<const NUM_BUFFERS: usize> {
    pub staging_entries: Vec<GPUChunkEntry>,
    pub buffer_offsets: [u64; NUM_BUFFERS],
    pub last_entry_offsets: [u64; NUM_BUFFERS],
    pub targets: Vec<BufferCopyTarget>,
}

impl<const NUM_BUFFERS: usize> StagingBufferMapping<NUM_BUFFERS> {
    pub const fn new() -> Self {
        Self {
            staging_entries: Vec::new(),
            buffer_offsets: [0; NUM_BUFFERS],
            last_entry_offsets: [0; NUM_BUFFERS],
            targets: Vec::new(),
        }
    }

    pub fn pop_target(&mut self) -> Option<(GPUChunkEntry, BufferCopyTarget)> {
        if self.targets.is_empty() {
            return None;
        }
        Some((
            self.staging_entries.pop().unwrap(),
            self.targets.pop().unwrap(),
        ))
    }

    pub fn push_to(&mut self, size: u64, allocation: MeshAllocation) {
        self.buffer_offsets[allocation.buffer_index] += size;
        let target = BufferCopyTarget {
            entry_offset: self.last_entry_offsets[allocation.buffer_index],
            size,
            allocation,
        };
        self.last_entry_offsets[allocation.buffer_index] = target.entry_offset + target.size;
        self.targets.push(target);
    }

    pub fn update_buffer_offsets(&mut self) {
        for i in (0..self.buffer_offsets.len()).rev() {
            self.buffer_offsets[i] = self.buffer_offsets[..i].iter().sum();
        }
    }

    pub fn push_to_staging(
        &mut self,
        chunks: &[Chunk],
        chunk_allocations: &mut Slap<
            IVec3,
            <MeshAllocator<NUM_BUFFERS> as VirtualMalloc>::Allocation,
        >,
    ) {
        for i in 0..self.targets.len() {
            let chunk = &chunks[i];
            let target = &self.targets[i];

            let mesh_alloc = target.allocation;
            let slab_index = chunk_allocations.insert(chunk.position, mesh_alloc);
            let staging_offset = self.buffer_offsets[mesh_alloc.buffer_index] + target.entry_offset;
            let header = GPUChunkEntryHeader::new(
                staging_offset as u32,
                mesh_alloc.offset as i32 - staging_offset as i32,
                target.size as u32,
                slab_index as u32,
                chunk.position,
            );
            let adjacent_blocks: [[[u32; PACKED_CHUNK_DIM]; CHUNK_DIM]; 3] = unsafe {
                *(chunk.adjacent_blocks.as_ptr()
                    as *const [[[u32; PACKED_CHUNK_DIM]; CHUNK_DIM]; 3])
            };
            self.staging_entries.push(GPUChunkEntry::new(
                header,
                adjacent_blocks,
                chunk.blocks,
            ));
        }
    }
}
