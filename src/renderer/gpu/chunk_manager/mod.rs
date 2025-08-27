mod chunk_compute;
mod chunk_manager;
mod chunk_render;

use crate::renderer::{DrawIndexedIndirectArgsA32, Index, Vertex};
pub use chunk_manager::ChunkManager;
use std::collections::HashMap;
use crate::renderer::gpu::MeshAllocation;

type BufferDrawArgs<const N: usize> = [HashMap<usize, DrawIndexedIndirectArgsA32>; N];

#[derive(Debug)]
struct MultiDrawInstruction {
    offset: usize,
    count: usize,
}


#[derive(Debug, Clone)]
struct BufferCopyMapping<const NUM_BUFFERS: usize> {
    pub target_buffers: [usize; NUM_BUFFERS],
    pub item_offsets: Vec<usize>,
    pub item_allocations: Vec<MeshAllocation>,
    pub target_indexes: Vec<usize>,
    last_inserted: usize,
}

impl<const NUM_BUFFERS: usize> BufferCopyMapping<NUM_BUFFERS> {
    pub const fn new() -> Self {
        Self {
            target_buffers: [0; NUM_BUFFERS],
            item_offsets: Vec::new(),
            item_allocations: Vec::new(),
            target_indexes: Vec::new(),
            last_inserted: 0,
        }
    }
    
    pub fn push_to(&mut self, target_buffer: usize, offset: usize, allocation: MeshAllocation) -> MeshAllocation {
        self.target_buffers[(target_buffer+1)..].iter_mut().for_each(|b| *b += offset);
        self.item_offsets.push(self.last_inserted);
        self.target_indexes.push(target_buffer);
        self.last_inserted = offset;
        self.item_allocations.push(allocation);
        self.get_mesh_alloc(self.item_offsets.len() - 1)
    }
    
    fn get_mesh_alloc(&self, index: usize) -> MeshAllocation {
        MeshAllocation {
            vertex_offset: self.item_offsets[index] as u32 * 4 * Vertex::size() as u32,
            index_offset: self.item_offsets[index] as u32 * 6 * size_of::<Index>() as u32,
            vertex_count: self.item_offsets[index] as u32 * 4,
            index_count: self.item_offsets[index] as u32 * 6,
        }
    }
}
