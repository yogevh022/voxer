
fn rotate_z(arr: ChunkBlocks) {
    for (var x: u32 = 0u; x < CHUNK_DIM_U16; x++) {
        for (var y: u32 = 0u; y < CHUNK_DIM_U16; y++) {
            rot_output[CHUNK_DIM_U16 - 1 - y][x] = arr[x][y];
        }
    }
}

fn rotate_y(arr: ChunkBlocks) {
    for (var x: u32 = 0u; x < CHUNK_DIM_U16; x++) {
        for (var y: u32 = 0u; y < CHUNK_DIM_U16; y++) {
            for (var z: u32 = 0u; z < CHUNK_DIM_U32; z++) {
                rot_output[CHUNK_DIM_U16 - 1 - z][y][x] = arr[x][y][z];
            }
        }
    }
}

fn calc_face_data(blocks: ChunkBlocks, face_axis: u32) {
    var index_count: u32 = 0u;
    var vertex_count: u32 = 0u;
    for (var x: u32 = 0u; x < CHUNK_DIM_U16 - 1u; x++) {
        for (var y: u32 = 0u; y < CHUNK_DIM_U16 - 1u; y++) {
            var z_blocks: array<u32, CHUNK_DIM_U16>;
            for (var z: u32 = 0u; z < CHUNK_DIM_U32; z++) {
                // x data no rotation
                let x_faces = blocks[x][y][z] ^ blocks[x+1u][y][z];
                let x_dirs = blocks[x][y][z] & (~blocks[x+1u][y][z]);
                // y data rotate cw on z
                let y_faces = blocks[y][x][z] ^ blocks[y+1u][x][z];
                let y_dirs = blocks[y][x][z] & (~blocks[y+1u][x][z]);
                // z data rotate cw on y
                // LOOP TWICE X 0-7 8-15
                z_blocks[z] |= ((blocks[z][y][x] >> x) & 1) << x;
                z_blocks[8u+z] |= ((blocks[8u+z][y][x] >> x) & 1) << x;
            }
            let z_faces = z_blocks[y][x] ^ blocks[z+1u][y][x];
            let z_dirs = z_blocks[y][x] & (~blocks[z+1u][y][x]);
            // z_arr logic
        }
        // y pos +1 code
    }
}