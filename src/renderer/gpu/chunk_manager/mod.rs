mod chunk_compute;
mod chunk_manager;
mod chunk_render;

use crate::renderer::{DrawIndexedIndirectArgsA32, DrawIndirectArgsA32};
use crate::renderer::gpu::{VMallocFirstFit, VMallocMultiBuffer, VirtualMalloc};
pub use chunk_manager::ChunkManager;
use std::collections::HashMap;
use wgpu::util::DrawIndirectArgs;

type BufferDrawArgs = HashMap<usize, DrawIndirectArgs>;
type MeshAllocationRequest = <VMallocFirstFit as VirtualMalloc>::AllocationRequest;

#[derive(Debug)]
struct MultiDrawInstruction {
    offset: usize,
    count: usize,
}
