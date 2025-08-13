fn bit_at(value: u32, index: u32) -> u32 {
    return (value >> index) & 1u;
}

fn rotate_z(arr: ChunkBlocks) {
    for (var x: u32 = 0u; x < CHUNK_DIM_U16; x += 1u) {
        for (var y: u32 = 0u; y < CHUNK_DIM_U16; y += 1u) {
            rot_output[y][CHUNK_DIM_U16 - 1 - x] = arr[x][y];
        }
    }
}

fn rotate_y(arr: ChunkBlocks) {
    for (var x: u32 = 0u; x < CHUNK_DIM_U16; x += 1u) {
        for (var y: u32 = 0u; y < CHUNK_DIM_U16; y += 1u) {
            for (var z: u32 = 0u; z < CHUNK_DIM_U32; z += 1u) {
                rot_output[x][y][z] = arr[x][y][z];
            }
        }
    }
}

fn calc_face_data(blocks: ChunkBlocks, face_axis: u32) {
    for (var x: u32 = 0u; x < CHUNK_DIM_U16 - 1u; x += 1u) {
        var arr_a: array<array<u32, CHUNK_DIM_U32>, CHUNK_DIM_U16> = blocks[x];
        var arr_b: array<array<u32, CHUNK_DIM_U32>, CHUNK_DIM_U16> = blocks[x+1u];
        for (var y: u32 = 0u; y < CHUNK_DIM_U16; y += 1u) {
            for (var z: u32 = 0u; z < CHUNK_DIM_U32; z += 1u) {
                chunk_face[face_axis][x].faces[y][z] = arr_a[y][z] ^ arr_b[y][z];
                chunk_face[face_axis][x].dirs[y][z] = arr_a[y][z] & (~arr_b[y][z]);
            }
        }
    }
}