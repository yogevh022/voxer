use bytemuck::{Pod, Zeroable};
use glam::IVec3;
use voxer_macros::ShaderType;
use crate::world::{VoxelChunkBlocks, CHUNK_DIM, CHUNK_DIM_HALF};

type ShaderAtomic<T> = T;

#[repr(C)]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUPackedIndirectArgsAtomic {
    draw: ShaderAtomic<u32>,
    _padding: u32,
    _padding1: u32,
    _padding2: u32,
    dispatch: GPUDispatchIndirectArgsAtomic,
}

impl GPUPackedIndirectArgsAtomic {
    pub fn new(draw: u32, dispatch: GPUDispatchIndirectArgsAtomic) -> Self {
        Self {
            draw,
            _padding: 0,
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
    pub negative_face_count: u32,
    // x: 10b,
    // y: 10b,
    // z: 10b,
    // free :1b,
    // meshing_flag: 1b,
    pub positive_face_count: u32,
    // x: 10b,
    // y: 10b,
    // z: 10b,
    // free :2b,
    pub face_alloc: u32,
}

impl GPUChunkMeshEntry {
    pub fn new(
        index: u32,
        negative_face_count: u32,
        positive_face_count: u32,
        face_alloc: u32,
    ) -> Self {
        Self {
            index,
            negative_face_count,
            positive_face_count,
            face_alloc,
        }
    }
}

#[repr(C, align(4))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUChunkMeshEntryWrite {
    entry: GPUChunkMeshEntry,
    index: u32,
}

impl GPUChunkMeshEntryWrite {
    pub fn new(entry: GPUChunkMeshEntry, index: u32) -> Self {
        Self { entry, index }
    }
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

#[repr(C, align(4))]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct GPUVoxelChunkHeader {
    pub(crate) index: u32,
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
}

impl GPUVoxelChunkHeader {
    pub fn new(chunk_index: u32, chunk_position: IVec3) -> Self {
        Self {
            index: chunk_index,
            chunk_x: chunk_position.x,
            chunk_y: chunk_position.y,
            chunk_z: chunk_position.z,
        }
    }
    
    pub(crate) fn position(&self) -> IVec3 {
        IVec3::new(self.chunk_x, self.chunk_y, self.chunk_z)
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

impl GPUVoxelChunk {
    pub fn new_uninit(header: GPUVoxelChunkHeader, blocks: VoxelChunkBlocks) -> Self {
        let gpu_content: GPUVoxelChunkContent = unsafe { std::mem::transmute(blocks) };
        let gpu_adj_content = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        Self {
            header,
            adj_content: gpu_adj_content,
            content: gpu_content,
        }
    }
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
