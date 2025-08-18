use crate::compute;
use crate::renderer::gpu::virtual_alloc::ChunkVMA;
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
    pub vertex_offset: u32, // in position not bytes
    pub index_offset: u32,  // in position not bytes
    pub vertex_count: u32,
    pub index_count: u32,
    pub slab_index: u32,      // 20
    _pad0: [u32; 3],          // pad to 32
    pub chunk_position: IVec3, // 44
    _pad3: u32,               // pad to 48
}

impl GPUChunkEntryHeader {
    pub fn new(
        vertex_offset: u32,
        index_offset: u32,
        vertex_count: u32,
        index_count: u32,
        slab_index: u32,
        chunk_position: IVec3,
    ) -> Self {
        Self {
            vertex_offset,
            index_offset,
            vertex_count,
            index_count,
            slab_index,
            _pad0: [0; 3],
            chunk_position,
            _pad3: 0,
        }
    }

    pub fn from_chunk_data(
        malloc: &mut ChunkVMA,
        chunk: &Chunk,
        chunk_position: IVec3,
        slab_index: u32,
    ) -> Self {
        let face_count = compute::chunk::face_count(&chunk.blocks);
        let vertex_count = face_count * 4;
        let index_count = face_count * 6;
        let vertex_offset = malloc.vertex.alloc(vertex_count).unwrap();
        let index_offset = malloc.index.alloc(index_count).unwrap();
        Self::new(
            vertex_offset as u32,
            index_offset as u32,
            vertex_count as u32,
            index_count as u32,
            slab_index,
            chunk_position,
        )
    }

    pub fn draw_indexed_indirect_args(&self) -> DrawIndexedIndirectArgs {
        DrawIndexedIndirectArgs {
            index_count: self.index_count,
            instance_count: 1,
            first_index: self.index_offset,
            base_vertex: 0, // vertices are indexed from 0, void and chunk offsets are baked into the indices
            first_instance: self.slab_index,
        }
    }
}
