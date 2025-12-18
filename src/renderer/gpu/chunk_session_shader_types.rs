use crate::renderer::gpu::vx_gpu_delta_vec::GpuIndexedItem;
use crate::world::{CHUNK_DIM, CHUNK_DIM_HALF, VoxelChunkAdjBlocks, VoxelChunkBlocks};
use bytemuck::{Pod, Zeroable};
use glam::{IVec3, UVec3};
use voxer_macros::ShaderType;

type ShaderAtomic<T> = T;

#[repr(C)]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUPackedIndirectArgsAtomic {
    draw: ShaderAtomic<u32>,
    _padding0: u32, // fixme _cpu_padding
    _padding1: u32,
    _padding2: u32,
    dispatch: GPUDispatchIndirectArgsAtomic,
}

impl GPUPackedIndirectArgsAtomic {
    pub fn new(draw: u32, dispatch: GPUDispatchIndirectArgsAtomic) -> Self {
        Self {
            draw,
            _padding0: 0,
            _padding1: 0,
            _padding2: 0,
            dispatch,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

#[repr(C)]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUDispatchIndirectArgsAtomic {
    x: ShaderAtomic<u32>,
    y: ShaderAtomic<u32>,
    z: ShaderAtomic<u32>,
}

impl GPUDispatchIndirectArgsAtomic {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
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
    // face_alloc: 31b,
    // meshing_flag: 1b,
}

// #[repr(C, align(4))]
// #[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
// pub struct GPUChunkMeshEntry {
//     pub index: u32,
//     pub negative_faces: u32,
//     // x: 10b,
//     // y: 10b,
//     // z: 10b,
//     // free :1b,
//     // meshing_flag: 1b,
//     pub positive_faces: u32,
//     // x: 10b,
//     // y: 10b,
//     // z: 10b,
//     // free :2b,
//     pub face_alloc: u32,
// }
//
// impl GPUChunkMeshEntry {
//     pub fn new(index: u32, negative_faces: u32, positive_faces: u32, face_alloc: u32) -> Self {
//         Self {
//             index,
//             negative_faces,
//             positive_faces,
//             face_alloc,
//         }
//     }
// }

impl GpuIndexedItem for GPUChunkMeshEntry {
    type WriteEntry = GPUChunkMeshEntryWrite;

    fn index(&self) -> usize {
        self.index as usize
    }

    fn init(mut self) -> Self {
        // meshing flag as true
        self.face_alloc |= 1 << 31;
        self
    }

    fn reused(mut self) -> Self {
        // meshing flag as false
        self.face_alloc &= !(1 << 31);
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
    blocks: [[[u32; CHUNK_DIM_HALF]; CHUNK_DIM]; CHUNK_DIM],
}

#[repr(C, align(4))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunkAdjContent {
    next_blocks: [[[u32; CHUNK_DIM_HALF]; CHUNK_DIM]; 3],
    prev_blocks: [[[u32; CHUNK_DIM_HALF]; CHUNK_DIM]; 3],
}

#[repr(C)] // pad-aligned to 16
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunkHeader {
    pub index: u32,
    _cpu_padding0: [u32; 3],
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
            _cpu_padding0: [0; 3],
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
    pub adj_content: VoxelChunkAdjBlocks, // 3072 bytes
    pub content: VoxelChunkBlocks,        // 8192 bytes
    pub header: GPUVoxelChunkHeader,      // 64 bytes
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
    // 11,280 bytes total
    pub header: GPUVoxelChunkHeader,          // 16 bytes
    pub adj_content: GPUVoxelChunkAdjContent, // 3072 bytes
    pub content: GPUVoxelChunkContent,        // 8192 bytes
}

#[repr(C, align(4))]
#[derive(ShaderType)]
pub struct GPUVoxelFaceData {
    // world_x + (world_z lower 12b) in instance_index
    word_a: u32,
    // voxel: 16b
    // top_left_R: 6b
    // top_left_G: 6b
    // face_id: 3b
    // 1b free
    word_b: u32,
    // face_y: 4b
    // top_left_B: 6b
    // top_left_AO: 2b
    // top_right_R: 6b
    // top_right_G: 6b
    // top_right_B: 6b
    // top_right_AO: 2b
    word_c: u32,
    // face_x: 4b
    // face_z: 4b
    // chunk_y: 8b
    // chunk_z_upper: 8b
    // bottom_left_R: 6b
    // bottom_left_AO: 2b
    word_d: u32,
    // bottom_left_G: 6b
    // bottom_left_B: 6b
    // bottom_right_R: 6b
    // bottom_right_G: 6b
    // bottom_right_B: 6b
    // bottom_right_AO: 2b
}
