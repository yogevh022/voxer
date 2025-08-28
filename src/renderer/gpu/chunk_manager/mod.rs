mod chunk_compute;
mod chunk_manager;
mod chunk_render;

use crate::renderer::DrawIndexedIndirectArgsA32;
use crate::renderer::gpu::{VMallocFirstFit, VMallocMultiBuffer, VirtualMalloc};
pub use chunk_manager::ChunkManager;
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
    pub buffer_offsets: [u64; NUM_BUFFERS],
    pub targets: Vec<BufferCopyTarget>,
}

impl<const NUM_BUFFERS: usize> StagingBufferMapping<NUM_BUFFERS> {
    pub const fn new() -> Self {
        Self {
            buffer_offsets: [0; NUM_BUFFERS],
            targets: Vec::new(),
        }
    }

    pub fn push_to(&mut self, size: usize, allocation: MeshAllocation) {
        self.buffer_offsets[allocation.buffer_index] += size as u64;
        let target = BufferCopyTarget {
            entry_offset: self
                .targets
                .last()
                .map(|t| t.entry_offset + t.size)
                .unwrap_or(0),
            size: size as u64,
            allocation,
        };
        self.targets.push(target);
    }

    pub fn update_buffer_offsets(&mut self) {
        for i in (0..self.buffer_offsets.len()).rev() {
            self.buffer_offsets[i] = self.buffer_offsets[..i].iter().sum();
        }
    }
}
