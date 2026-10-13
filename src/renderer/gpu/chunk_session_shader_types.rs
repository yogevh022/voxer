use crate::renderer::gpu::vx_gpu_sync_vec::GpuIndexedItem;
use crate::world::{CHUNK_DIM, VoxelChunkAdjBlocks, VoxelChunkBlocks};
use bytemuck::{Pod, Zeroable};
use glam::{IVec3, UVec3};
use voxer_macros::ShaderType;

type ShaderAtomic<T> = T;

#[repr(C)]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUIndirectArgsAtomic {
    draw: ShaderAtomic<u32>,
    _padding0: u32, // cpu and gpu padding
    _padding1: u32,
    _padding2: u32,
}

impl GPUIndirectArgsAtomic {
    pub fn new(draw: u32) -> Self {
        Self {
            draw,
            _padding0: 0,
            _padding1: 0,
            _padding2: 0,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

#[repr(C, align(16))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUDrawIndirectArgs {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
}

#[repr(C, align(4))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUChunkMeshEntry {
    pub index: u32,
    pub face_alloc: u32,
}

impl GpuIndexedItem for GPUChunkMeshEntry {
    type WriteEntry = GPUChunkMeshEntryWrite;

    fn index(&self) -> usize {
        self.index as usize
    }

    fn init(self) -> Self {
        self
    }

    fn reused(self) -> Self {
        self
    }

    fn write(self, index: usize) -> Self::WriteEntry {
        Self::WriteEntry {
            entry: self,
            index: index as u32,
        }
    }
}

#[repr(C, align(4))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUChunkMeshEntryWrite {
    pub entry: GPUChunkMeshEntry,
    pub index: u32,
}

#[repr(C, align(4))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunkContent {
    blocks: [[[u16; CHUNK_DIM]; CHUNK_DIM]; CHUNK_DIM],
}

#[repr(C, align(4))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunkContentWithAdj {
    blocks: [[[u16; CHUNK_DIM + 2]; CHUNK_DIM + 2]; CHUNK_DIM + 2],
}

#[repr(C, align(4))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunkAdjContent {
    next_blocks: [[[u16; CHUNK_DIM]; CHUNK_DIM]; 3],
    prev_blocks: [[[u16; CHUNK_DIM]; CHUNK_DIM]; 3],
}

#[repr(C, align(4))] // pad-aligned to 16
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunkHeader {
    pub index: u32,
    pub face_alloc: u32,
    _cpu_padding0: [u32; 2],
    pub faces_positive: UVec3,
    _cpu_padding1: u32,
    pub faces_negative: UVec3,
    _cpu_padding2: u32,
    pub position: IVec3,
    _cpu_padding3: u32,
}

impl GPUVoxelChunkHeader {
    pub fn new(index: u32, position: IVec3) -> Self {
        Self {
            index,
            face_alloc: 0,
            _cpu_padding0: [0; 2],
            faces_positive: UVec3::ZERO,
            _cpu_padding1: 0,
            faces_negative: UVec3::ZERO,
            _cpu_padding2: 0,
            position,
            _cpu_padding3: 0,
        }
    }
}

#[repr(C, align(4))]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct CPUVoxelChunk {
    // 11,328 bytes total
    pub header: GPUVoxelChunkHeader,      // 64 bytes
    pub adj_content: VoxelChunkAdjBlocks, // 3072 bytes
    pub content: VoxelChunkBlocks,        // 8192 bytes
}

impl CPUVoxelChunk {
    pub fn new(header: GPUVoxelChunkHeader, blocks: VoxelChunkBlocks) -> Self {
        Self {
            header,
            adj_content: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
            content: blocks,
        }
    }
}

#[repr(C, align(4))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunk {
    // 11,328 bytes total
    pub header: GPUVoxelChunkHeader,          // 64 bytes
    pub adj_content: GPUVoxelChunkAdjContent, // 3072 bytes
    pub content: GPUVoxelChunkContent,        // 8192 bytes
}

#[repr(C, align(4))]
#[derive(ShaderType)]
pub struct GPUVoxelFaceData {
    word_a: u32,
    // voxel: 16b
    // face_x: 4b
    // face_y: 4b
    // face_z: 4b
    // face_id: 3b
    // 1b free
}
