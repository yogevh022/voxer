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

#[derive(Debug)]
struct MultiDrawInstruction {
    offset: usize,
    count: usize,
}
