struct LayerFaceData {
    faces: array<array<u32, CHUNK_DIM_U32>, CHUNK_DIM_U16>,
    dirs: array<array<u32, CHUNK_DIM_U32>, CHUNK_DIM_U16>,
}

struct FaceMask {
    face_bit: u32,
    dir_bit: u32,
}

alias ChunkBlocks = array<array<array<u32, CHUNK_DIM_U32>, CHUNK_DIM_U16>, CHUNK_DIM_U16>; // wgsl has no u16 :D
