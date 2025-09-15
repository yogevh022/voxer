
const CFG_VAO_FACTOR: f32 = 0.35;


const VCHUNK_DIM: u32 = 16u;
const VCHUNK_DIM_HALF: u32 = 8u;

const VCHUNK_ENTRY_BYTES: u32 = VCHUNK_ENTRY_HEADER_BYTES + VCHUNK_ADJ_BLOCKS_BYTES + VCHUNK_BLOCKS_BYTES;
struct VoxelChunkEntry {
    header: VoxelChunkEntryHeader,
    adjacent_blocks: VoxelChunkAdjacentBlocks,
    blocks: VoxelChunkBlocks,
}
const VCHUNK_ENTRY_HEADER_BYTES: u32 = 32u;
struct VoxelChunkEntryHeader {
    position: vec3<i32>,
    slab_index: u32, // 20
    buffer_data: VoxelChunkEntryBufferData,
    // padded to 32
}
struct VoxelChunkEntryBufferData {
    offset: u32,
    face_count: u32,
}

struct VoxelFaceData {
    position__fid__illum__ocl: u32,
    // position 12b
    // face id 3b
    // illumination 5b
    // occlusion count 8b
    // 4 free
    voxel_type: u32,
    // voxel_type 16b
    chunk_translation: vec3<i32>,
}

const VCHUNK_BLOCKS_BYTES: u32 = VCHUNK_DIM * VCHUNK_DIM * VCHUNK_DIM_HALF * 4; // u32
alias VoxelChunkBlocks = array<array<array<u32, VCHUNK_DIM_HALF>, VCHUNK_DIM>, VCHUNK_DIM>;

const VCHUNK_ADJ_BLOCKS_BYTES: u32 = VCHUNK_DIM * VCHUNK_DIM * 3 * 4; // 3 axes, u32
alias VoxelChunkAdjacentBlocks = array<array<array<u32, VCHUNK_DIM_HALF>, VCHUNK_DIM>, 3>;
