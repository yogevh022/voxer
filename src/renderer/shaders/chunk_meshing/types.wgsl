const CHUNK_DIM: u32 = 16u;
const CHUNK_DIM_HALF: u32 = 8u;
const TILE_DIM: f32 = 0.5;

const VOID_OFFSET: u32 = 8u;
const MAX_VERTICES_PER_THREAD: u32 = ((4u * 3u) * CHUNK_DIM) + VOID_OFFSET;
const MAX_INDICES_PER_THREAD: u32 = ((6u * 3u) * CHUNK_DIM) + VOID_OFFSET;

const CHUNK_HEADER_BYTES: u32 = 48u;
const CHUNK_BLOCKS_BYTES: u32 = CHUNK_DIM * CHUNK_DIM * CHUNK_DIM_HALF * 4; // u32
const CHUNK_ENTRY_BYTES: u32 = CHUNK_HEADER_BYTES + CHUNK_BLOCKS_BYTES;

const MAX_CHUNK_ENTRIES: u32 = 12288u / CHUNK_ENTRY_BYTES;

struct ChunkEntryHeader {
    staging_offset: u32,
    target_offset_delta: i32,
    face_count: u32,
    _pad: u32,
    slab_index: u32, // 20
    // padded to 32
    chunk_position: vec3<i32>,
}

struct ChunkEntry {
    header: ChunkEntryHeader,
    blocks: ChunkBlocks,
}

alias ChunkBlocks = array<array<array<u32, CHUNK_DIM_HALF>, CHUNK_DIM>, CHUNK_DIM>; // wgsl has no u16 :D
alias ChunkEntryBuffer = array<ChunkEntry, MAX_CHUNK_ENTRIES>;
