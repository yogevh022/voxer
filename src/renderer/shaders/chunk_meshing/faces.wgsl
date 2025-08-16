
fn write_faces_x(
    packed_faces: u32,
    packed_dirs: u32,
    index_count: ptr<function, u32>,
    vertex_count: ptr<function, u32>,
    x: u32,
    y: u32,
    z: u32,
) {
    let temp_uv: vec2<f32> = vec2<f32>(0.0, 0.0);

    let x_f32 = f32(x);
    let y_f32 = f32(y);

    // logic for both u16s packed into the u32
    for (var n = 0u; n < 2u; n++) {
        let z_f32 = f32(z - n);
        let bit_index = (16u << n) - 1u;
        let draw_face: bool = ((packed_faces >> bit_index) & 1u) != 0u;
        let face_dir: bool = ((packed_dirs >> bit_index) & 1u) != 0u;
        let i_index = chunk_index_offset + (*index_count);
        let v_index = chunk_vertex_offset + (*vertex_count);

        quad_indices(draw_face, i_index, v_index);
        plus_x_vertices(draw_face && face_dir, v_index, temp_uv ,x_f32 ,y_f32, z_f32);
        minus_x_vertices(draw_face && !face_dir, v_index, temp_uv, x_f32, y_f32, z_f32);
        (*index_count) += select(0u, 6u, draw_face);
        (*vertex_count) += select(0u, 4u, draw_face);
    }
}

fn write_faces_y(
    packed_faces: u32,
    packed_dirs: u32,
    index_count: ptr<function, u32>,
    vertex_count: ptr<function, u32>,
    x: u32,
    y: u32,
    z: u32,
) {
    let temp_uv: vec2<f32> = vec2<f32>(0.0, 0.0);

    let x_f32 = f32(x);
    let y_f32 = f32(y);

    // logic for both u16s packed into the u32
    for (var n = 0u; n < 2u; n ++) {
        let z_f32 = f32(z - n);
        let draw_face: bool = ((packed_faces >> ((16u << n) - 1u)) & 1u) != 0u;
        let face_dir: bool = ((packed_dirs >> ((16u << n) - 1u)) & 1u) != 0u;
        let i_index = chunk_index_offset + (*index_count);
        let v_index = chunk_vertex_offset + (*vertex_count);

        quad_indices(draw_face, i_index, v_index);
        plus_y_vertices(draw_face && face_dir, v_index, temp_uv ,x_f32 ,y_f32, z_f32);
        minus_y_vertices(draw_face && !face_dir, v_index, temp_uv, x_f32, y_f32, z_f32);
        (*index_count) += select(0u, 6u, draw_face);
        (*vertex_count) += select(0u, 4u, draw_face);
    }
}

fn calc_face_data(blocks: ChunkBlocks) {
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


                write_faces_x(
                    x_faces,
                    x_dirs,
                    &index_count,
                    &vertex_count,
                    x,
                    y,
                    z*2,
                );

                write_faces_y(
                    y_faces,
                    y_dirs,
                    &index_count,
                    &vertex_count,
                    x,
                    y,
                    z*2,
                );
            }
        }
    }
}