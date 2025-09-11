use crate::renderer::DrawIndexedIndirectArgsA32;
use crate::world::types::{CHUNK_DIM, ChunkBlocks, PACKED_CHUNK_DIM};
use bytemuck::{Pod, Zeroable};
use glam::IVec3;

type GPUPackedBlockPair = u32;
type GPUChunkBlocks = [[[GPUPackedBlockPair; PACKED_CHUNK_DIM]; CHUNK_DIM]; CHUNK_DIM];
type GPUChunkAdjacentBlocks = [[[GPUPackedBlockPair; PACKED_CHUNK_DIM]; CHUNK_DIM]; 3];

#[repr(C, align(8))]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUChunkEntryBufferData {
    pub offset: u32,
    pub face_count: u32,
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUChunkEntryHeader {
    pub position: IVec3,                      // 0-11
    pub slab_index: u32,                      // 12-15
    pub buffer_data: GPUChunkEntryBufferData, // 16-23
    _padding: [u32; 2],                       // 23-31
}

impl GPUChunkEntryHeader {
    pub fn new(offset: u32, face_count: u32, slab_index: u32, chunk_position: IVec3) -> Self {
        let buffer_data = GPUChunkEntryBufferData { face_count, offset };
        Self {
            buffer_data,
            slab_index,
            position: chunk_position,
            _padding: [0; 2],
        }
    }

    pub fn draw_indexed_indirect_args(&self) -> DrawIndexedIndirectArgsA32 {
        DrawIndexedIndirectArgsA32::new(
            self.buffer_data.face_count * 6,
            1,
            self.buffer_data.offset * 6,
            0,
            self.slab_index,
        )
    }
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUChunkEntry {
    pub header: GPUChunkEntryHeader,
    pub adjacent_blocks: GPUChunkAdjacentBlocks,
    pub blocks: GPUChunkBlocks,
}

impl GPUChunkEntry {
    pub fn new(
        header: GPUChunkEntryHeader,
        adjacent_blocks: GPUChunkAdjacentBlocks,
        blocks: ChunkBlocks,
    ) -> Self {
        let gpu_blocks: GPUChunkBlocks = unsafe { std::mem::transmute(blocks) };
        Self {
            header,
            adjacent_blocks,
            blocks: gpu_blocks,
        }
    }
}
