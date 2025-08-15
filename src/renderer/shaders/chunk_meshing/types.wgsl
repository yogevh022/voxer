struct LayerFaceData {
    faces: array<array<u32, CHUNK_DIM_U32>, CHUNK_DIM_U16>,
    dirs: array<array<u32, CHUNK_DIM_U32>, CHUNK_DIM_U16>,
}

alias ChunkBlocks = array<array<array<u32, CHUNK_DIM_U32>, CHUNK_DIM_U16>, CHUNK_DIM_U16>; // wgsl has no u16 :D
