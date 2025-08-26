
fn write_faces_x(
    packed_faces: u32,
    packed_dirs: u32,
    index_array: ptr<function, array<Index, MAX_INDICES_PER_THREAD>>,
    vertex_array: ptr<function, array<Vertex, MAX_VERTICES_PER_THREAD>>,
    local_index_count: ptr<function, u32>,
    local_vertex_count: ptr<function, u32>,
    x: u32,
    y: u32,
    z: u32,
) {
    let temp_uv: vec2<f32> = vec2<f32>(0.0, 0.0);

    let x_f32 = f32(x);
    let y_f32 = f32(y);

    // logic for both u16s packed into the u32
    for (var n = 2u; n >= 1u; n--) {
        let z_f32 = f32(z - n);
        let bit_index = (16u << (2u - n)) - 1u;
        let draw_face = (packed_faces >> bit_index) & 1u;
        let face_dir = (packed_dirs >> bit_index) & 1u;
        let i_index = draw_face * (*local_index_count);
        let v_index = draw_face * (*local_vertex_count);

//        quad_indices(index_array, i_index, v_index);
//        minus_x_vertices(vertex_array, face_dir * v_index, temp_uv ,x_f32 ,y_f32, z_f32);
//        plus_x_vertices(vertex_array, (1u ^ face_dir) * v_index, temp_uv, x_f32, y_f32, z_f32);
        (*local_index_count) += 6u * draw_face;
        (*local_vertex_count) += 4u * draw_face;
    }
}

fn write_faces_y(
    packed_faces: u32,
    packed_dirs: u32,
    index_array: ptr<function, array<Index, MAX_INDICES_PER_THREAD>>,
    vertex_array: ptr<function, array<Vertex, MAX_VERTICES_PER_THREAD>>,
    local_index_count: ptr<function, u32>,
    local_vertex_count: ptr<function, u32>,
    x: u32,
    y: u32,
    z: u32,
) {
    let temp_uv: vec2<f32> = vec2<f32>(0.0, 0.0);

    let x_f32 = f32(x);
    let y_f32 = f32(y);

    // logic for both u16s packed into the u32
    for (var n = 2u; n >= 1u; n--) {
        let z_f32 = f32(z - n);
        let bit_index = (16u << (2u - n)) - 1u;
        let draw_face = (packed_faces >> bit_index) & 1u;
        let face_dir = (packed_dirs >> bit_index) & 1u;
        let i_index = draw_face * (*local_index_count);
        let v_index = draw_face * (*local_vertex_count);

        quad_indices(index_array, i_index, v_index);
        plus_y_vertices(vertex_array, face_dir * v_index, temp_uv ,x_f32 ,y_f32, z_f32);
        minus_y_vertices(vertex_array, (1u ^ face_dir) * v_index, temp_uv, x_f32, y_f32, z_f32);
        (*local_index_count) += 6u * draw_face;
        (*local_vertex_count) += 4u * draw_face;
    }
}

fn write_faces_z(
    packed_faces: u32,
    packed_dirs: u32,
    index_array: ptr<function, array<Index, MAX_INDICES_PER_THREAD>>,
    vertex_array: ptr<function, array<Vertex, MAX_VERTICES_PER_THREAD>>,
    local_index_count: ptr<function, u32>,
    local_vertex_count: ptr<function, u32>,
    x: u32,
    y: u32,
    z: u32,
) {
    let temp_uv: vec2<f32> = vec2<f32>(0.0, 0.0);

    let x_f32 = f32(x);
    let y_f32 = f32(y);

    // logic for both u16s packed into the u32
    for (var n = 2u; n >= 1u; n--) {
        let z_f32 = f32(z - n);
        let bit_index = (16u << (2u - n)) - 1u;
        let draw_face = (packed_faces >> bit_index) & 1u;
        let face_dir = (packed_dirs >> bit_index) & 1u;
        let i_index = draw_face * (*local_index_count);
        let v_index = draw_face * (*local_vertex_count);

        quad_indices(index_array, i_index, v_index);
        plus_z_vertices(vertex_array, face_dir * v_index, temp_uv ,x_f32 ,y_f32, z_f32);
        minus_z_vertices(vertex_array, (1u ^ face_dir) * v_index, temp_uv, x_f32, y_f32, z_f32);
        (*local_index_count) += 6u * draw_face;
        (*local_vertex_count) += 4u * draw_face;
    }
}

fn mesh_chunk_position(chunk_index: u32, x: u32, y: u32) {
    let chunk = chunk_entries[chunk_index];
    var vertex_array: array<Vertex, MAX_VERTICES_PER_THREAD>;
    var index_array: array<Index, MAX_INDICES_PER_THREAD>;
    var local_vertex_count: u32 = VOID_OFFSET;
    var local_index_count: u32 = VOID_OFFSET;
    for (var z: u32 = 0u; z < CHUNK_DIM_HALF; z++) {
        let current: u32 = chunk.blocks[x][y][z];
        let next_x: u32 = select(0u, chunk.blocks[x+1u][y][z], x < (CHUNK_DIM - 1));
        let x_faces = current ^ next_x;
        let x_dirs = current & (~next_x);
        write_faces_x(
            x_faces,
            x_dirs,
            &index_array,
            &vertex_array,
            &local_index_count,
            &local_vertex_count,
            x,
            y,
            (z+1u)*2,
        );

        let next_y: u32 = select(0u, chunk.blocks[x][y+1u][z], y < (CHUNK_DIM - 1));
        let y_faces = current ^ next_y;
        let y_dirs = current & (~next_y);
//        write_faces_y(
//            y_faces,
//            y_dirs,
//            &index_array,
//            &vertex_array,
//            &local_index_count,
//            &local_vertex_count,
//            x,
//            y,
//            (z+1u)*2,
//        );

        let current_z_a = current & 0xFFFFu;
        let current_z_b = current >> 16u;
        var z_faces = current_z_a ^ current_z_b;
        var z_dirs = current_z_a & (~current_z_b);

        let next_z: u32 = select(0u, chunk.blocks[x][y][z + 1u], z < (CHUNK_DIM_HALF - 1));
        let next_z_a = next_z & 0xFFFFu;
        z_faces |= (current_z_b ^ next_z_a) << 16u;
        z_dirs |= (current_z_b & (~next_z_a)) << 16u;
//        write_faces_z(
//            z_faces,
//            z_dirs,
//            &index_array,
//            &vertex_array,
//            &local_index_count,
//            &local_vertex_count,
//            x,
//            y,
//            (z+1u)*2,
//        );
    }
    let v_count_no_offset = local_vertex_count - VOID_OFFSET;
    let i_count_no_offset = local_index_count - VOID_OFFSET;
    let vertex_offset: u32 = atomicAdd(&vertex_count, v_count_no_offset);
    let index_offset: u32 = atomicAdd(&index_count, i_count_no_offset);
//    for (var i = 0u; i < v_count_no_offset; i++) {
//        staging_vertex_buffer[vertex_offset + i] = vertex_array[VOID_OFFSET + i];
//    }
//    for (var i = 0u; i < i_count_no_offset; i++) {
//        staging_index_buffer[index_offset + i] = (index_array[VOID_OFFSET + i] + vertex_offset) - VOID_OFFSET;
//    }
    for (var i = 0u; i < v_count_no_offset; i++) {
        staging_vertex_buffer[vertex_offset] = Vertex(vec3<f32>(0.0,0.0,0.0), vec2<f32>(0.0,0.0));//vertex_array[VOID_OFFSET + i];
    }
    for (var i = 0u; i < i_count_no_offset; i++) {
        staging_index_buffer[index_offset] = 0u;//(index_array[VOID_OFFSET + i] + vertex_offset) - VOID_OFFSET;
    }
}
