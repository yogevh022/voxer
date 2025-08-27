mod chunk_compute;
mod chunk_render;
mod chunk_manager;

use std::collections::HashMap;
pub use chunk_manager::ChunkManager;
use crate::renderer::DrawIndexedIndirectArgsA32;

type BufferDrawArgs<const N: usize> = [HashMap<usize, DrawIndexedIndirectArgsA32>; N];

#[derive(Debug)]
struct MultiDrawInstruction {
    offset: usize,
    count: usize,
}