mod chunk_compute;
mod chunk_render;
mod chunk_manager;

use std::collections::HashMap;
use wgpu::util::DrawIndexedIndirectArgs;
pub use chunk_manager::ChunkManager;

type BufferDrawArgs<const N: usize> = [HashMap<usize, DrawIndexedIndirectArgs>; N];

#[derive(Debug)]
struct MultiDrawInstruction {
    offset: usize,
    count: usize,
}