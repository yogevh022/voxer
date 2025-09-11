const CHUNK_DIM: u32 = 16u;
const CHUNK_DIM_HALF: u32 = 8u;
const TILE_DIM: f32 = 0.5;

const VAO_FACTOR: f32 = 0.35;

const VOID_OFFSET: u32 = 8u;
const MAX_VERTICES_PER_THREAD: u32 = ((4u * 3u) * CHUNK_DIM) + VOID_OFFSET;
const MAX_INDICES_PER_THREAD: u32 = ((6u * 3u) * CHUNK_DIM) + VOID_OFFSET;

const CHUNK_HEADER_BYTES: u32 = 32u;
const CHUNK_BLOCKS_BYTES: u32 = CHUNK_DIM * CHUNK_DIM * CHUNK_DIM_HALF * 4; // u32
const CHUNK_ADJ_BLOCKS_BYTES: u32 = CHUNK_DIM * CHUNK_DIM * 3 * 4; // 3 axes, u32
const CHUNK_ENTRY_BYTES: u32 = CHUNK_HEADER_BYTES + CHUNK_ADJ_BLOCKS_BYTES+ CHUNK_BLOCKS_BYTES;

const MAX_CHUNK_ENTRIES: u32 = 12288u / CHUNK_ENTRY_BYTES;

struct ChunkEntryBufferData {
    offset: u32,
    face_count: u32,
}

struct ChunkEntryHeader {
    position: vec3<i32>,
    slab_index: u32, // 20
    buffer_data: ChunkEntryBufferData,
    // padded to 32
}

struct ChunkEntry {
    header: ChunkEntryHeader,
    adjacent_blocks: ChunkAdjacentBlocks,
    blocks: ChunkBlocks,
}

alias ChunkBlocks = array<array<array<u32, CHUNK_DIM_HALF>, CHUNK_DIM>, CHUNK_DIM>;
alias ChunkAdjacentBlocks = array<array<array<u32, CHUNK_DIM_HALF>, CHUNK_DIM>, 3>;
alias ChunkEntryBuffer = array<ChunkEntry, MAX_CHUNK_ENTRIES>;
