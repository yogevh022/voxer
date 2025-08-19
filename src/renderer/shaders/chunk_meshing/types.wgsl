const CHUNK_DIM: u32 = 16u;
const CHUNK_DIM_HALF: u32 = 8u;
const TILE_DIM: f32 = 0.5;

const MAX_VERTICES_PER_THREAD = (4u + 4u + 4u) * CHUNK_DIM;
const MAX_INDICES_PER_THREAD = (6u + 6u + 6u) * CHUNK_DIM;

const CHUNK_HEADER_BYTES: u32 = 48u;
const CHUNK_BLOCKS_BYTES: u32 = CHUNK_DIM * CHUNK_DIM * CHUNK_DIM_HALF * 4; // u32
const CHUNK_ENTRY_BYTES: u32 = CHUNK_HEADER_BYTES + CHUNK_BLOCKS_BYTES;

const MAX_CHUNK_ENTRIES: u32 = (MAX_BUFFER - 16u) / CHUNK_ENTRY_BYTES;

struct ChunkEntryBuffer {
    count: u32,
    // padded to 16
    chunks: array<ChunkEntry, MAX_CHUNK_ENTRIES>,
}

struct ChunkEntryHeader {
    vertex_offset: u32,
    index_offset: u32,
    vertex_count: u32,
    index_count: u32,
    slab_index: u32, // 20
    // padded to 32
    chunk_position: vec3<i32>,
}

struct ChunkEntry {
    header: ChunkEntryHeader,
    blocks: ChunkBlocks,
}

alias ChunkBlocks = array<array<array<u32, CHUNK_DIM_HALF>, CHUNK_DIM>, CHUNK_DIM>; // wgsl has no u16 :D
