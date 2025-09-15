mod chunk_compute;
mod chunk_manager;
mod chunk_render;

use crate::renderer::DrawIndirectArgsDX12;
use crate::renderer::gpu::{VMallocFirstFit, VirtualMalloc};
pub use chunk_manager::ChunkManager;
use std::collections::HashMap;

type BufferDrawArgs = HashMap<usize, DrawIndirectArgsDX12>;
type MeshAllocationRequest = <VMallocFirstFit as VirtualMalloc>::AllocationRequest;

#[derive(Debug)]
struct MultiDrawInstruction {
    offset: usize,
    count: usize,
}
