use crate::renderer::{DrawIndexedIndirectArgsA32, Index, Vertex};
use crate::world::types::{CHUNK_DIM, ChunkBlocks, PACKED_CHUNK_DIM};
use bytemuck::{Pod, Zeroable};
use glam::IVec3;

type GPUPackedBlockPair = u32;
type GPUChunkBlocks = [[[GPUPackedBlockPair; PACKED_CHUNK_DIM]; CHUNK_DIM]; CHUNK_DIM];

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUChunkEntryBufferData {
    pub staging_offset: u32,
    pub target_offset_delta: i32,
    pub face_count: u32,
    _padding: u32,
}

impl GPUChunkEntryBufferData {
    pub fn new(face_count: u32, staging_offset: u32, target_offset_delta: i32) -> Self {
        Self {
            staging_offset,
            target_offset_delta,
            face_count,
            _padding: 0,
        }
    }
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUChunkEntry {
    pub header: GPUChunkEntryHeader,
    pub blocks: GPUChunkBlocks,
}

impl GPUChunkEntry {
    pub fn new(header: GPUChunkEntryHeader, blocks: ChunkBlocks) -> Self {
        let gpu_blocks: GPUChunkBlocks = unsafe { std::mem::transmute(blocks) };
        Self {
            header,
            blocks: gpu_blocks,
        }
    }
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUChunkEntryHeader {
    pub buffer_data: GPUChunkEntryBufferData, // 16
    pub slab_index: u32,                      // 20
    _pad0: [u32; 3],                          // pad to 32
    pub chunk_position: IVec3,                // 44
    _pad3: u32,                               // pad to 48
}

impl GPUChunkEntryHeader {
    pub fn new(
        staging_offset: u32,
        target_offset_delta: i32,
        face_count: u32,
        slab_index: u32,
        chunk_position: IVec3,
    ) -> Self {
        let buffer_data =
            GPUChunkEntryBufferData::new(face_count, staging_offset, target_offset_delta);
        Self {
            buffer_data,
            slab_index,
            _pad0: [0; 3],
            chunk_position,
            _pad3: 0,
        }
    }

    pub fn draw_indexed_indirect_args(&self) -> DrawIndexedIndirectArgsA32 {
        DrawIndexedIndirectArgsA32::new(
            self.buffer_data.face_count * 6,
            1,
            self.buffer_data.staging_offset * 6,
            0, // vertices are indexed from 0, void and chunk offsets are baked into the indices
            self.slab_index,
        )
    }
}
