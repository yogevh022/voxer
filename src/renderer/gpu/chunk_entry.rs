use crate::renderer::gpu::VoxerMultiBufferMeshAllocation;
use crate::world::types::{CHUNK_DIM, Chunk, ChunkBlocks, PACKED_CHUNK_DIM};
use bytemuck::{Pod, Zeroable};
use glam::IVec3;
use std::ops::Deref;
use wgpu::util::DrawIndexedIndirectArgs;

type GPUPackedBlockPair = u32;
type GPUChunkBlocks = [[[GPUPackedBlockPair; PACKED_CHUNK_DIM]; CHUNK_DIM]; CHUNK_DIM];

#[derive(Debug)]
pub struct GPUChunkEntryBuffer(Vec<GPUChunkEntry>);

impl GPUChunkEntryBuffer {
    pub fn new(size: usize) -> Self {
        Self(Vec::with_capacity(size))
    }

    pub fn insert(&mut self, header: GPUChunkEntryHeader, blocks: ChunkBlocks) {
        let entry = GPUChunkEntry::new(header, blocks);
        self.0.push(entry);
    }
}

impl Deref for GPUChunkEntryBuffer {
    type Target = [GPUChunkEntry];

    fn deref(&self) -> &Self::Target {
        &self.0
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
    pub allocation: VoxerMultiBufferMeshAllocation, // 16
    pub slab_index: u32,                            // 20
    _pad0: [u32; 3],                                // pad to 32
    pub chunk_position: IVec3,                      // 44
    _pad3: u32,                                     // pad to 48
}

impl GPUChunkEntryHeader {
    pub fn new(
        allocation: VoxerMultiBufferMeshAllocation,
        slab_index: u32,
        chunk_position: IVec3,
    ) -> Self {
        Self {
            allocation,
            slab_index,
            _pad0: [0; 3],
            chunk_position,
            _pad3: 0,
        }
    }

    pub fn draw_indexed_indirect_args(&self) -> DrawIndexedIndirectArgs {
        DrawIndexedIndirectArgs {
            index_count: self.allocation.index_size as u32,
            instance_count: 1,
            first_index: self.allocation.index_offset as u32,
            base_vertex: 0, // vertices are indexed from 0, void and chunk offsets are baked into the indices
            first_instance: self.slab_index,
        }
    }
}
