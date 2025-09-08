
fn write_faces_x(
    packed_faces: u32,
    packed_dirs: u32,
    index_array: ptr<function, array<Index, MAX_INDICES_PER_THREAD>>,
    vertex_array: ptr<function, array<Vertex, MAX_VERTICES_PER_THREAD>>,
    local_count: ptr<function, u32>,
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
        let i_index = draw_face * ((*local_count * 6u) + VOID_OFFSET);
        let v_index = draw_face * ((*local_count * 4u) + VOID_OFFSET);

        quad_indices(index_array, i_index, v_index);
        minus_x_vertices(vertex_array, face_dir * v_index, temp_uv ,x_f32 ,y_f32, z_f32);
        plus_x_vertices(vertex_array, (1u ^ face_dir) * v_index, temp_uv, x_f32, y_f32, z_f32);
        (*local_count) += 1u * draw_face;
    }
}

fn write_faces_y(
    packed_faces: u32,
    packed_dirs: u32,
    index_array: ptr<function, array<Index, MAX_INDICES_PER_THREAD>>,
    vertex_array: ptr<function, array<Vertex, MAX_VERTICES_PER_THREAD>>,
    local_count: ptr<function, u32>,
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
        let i_index = draw_face * ((*local_count * 6u) + VOID_OFFSET);
        let v_index = draw_face * ((*local_count * 4u) + VOID_OFFSET);

        quad_indices(index_array, i_index, v_index);
        plus_y_vertices(vertex_array, face_dir * v_index, temp_uv ,x_f32 ,y_f32, z_f32);
        minus_y_vertices(vertex_array, (1u ^ face_dir) * v_index, temp_uv, x_f32, y_f32, z_f32);
        (*local_count) += 1u * draw_face;
    }
}

fn write_faces_z(
    packed_faces: u32,
    packed_dirs: u32,
    index_array: ptr<function, array<Index, MAX_INDICES_PER_THREAD>>,
    vertex_array: ptr<function, array<Vertex, MAX_VERTICES_PER_THREAD>>,
    local_count: ptr<function, u32>,
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
        let i_index = draw_face * ((*local_count * 6u) + VOID_OFFSET);
        let v_index = draw_face * ((*local_count * 4u) + VOID_OFFSET);

        quad_indices(index_array, i_index, v_index);
        plus_z_vertices(vertex_array, face_dir * v_index, temp_uv ,x_f32 ,y_f32, z_f32);
        minus_z_vertices(vertex_array, (1u ^ face_dir) * v_index, temp_uv, x_f32, y_f32, z_f32);
        (*local_count) += 1u * draw_face;
    }
}

fn mesh_chunk_position(chunk_index: u32, x: u32, y: u32) {
    let chunk = chunk_entries[chunk_index];
    var vertex_array: array<Vertex, MAX_VERTICES_PER_THREAD>;
    var index_array: array<Index, MAX_INDICES_PER_THREAD>;
    var local_count: u32 = 0u;
    for (var z: u32 = 0u; z < CHUNK_DIM_HALF; z++) {
        let current: u32 = chunk.blocks[x][y][z];
        let safe_xp = min(x+1, CHUNK_DIM - 1);
        let next_x: u32 = select(chunk.adjacent_blocks[0u][y][z], chunk.blocks[safe_xp][y][z], x < (CHUNK_DIM - 1));
        let x_faces = current ^ next_x;
        let x_dirs = current & (~next_x);
        write_faces_x(
            x_faces,
            x_dirs,
            &index_array,
            &vertex_array,
            &local_count,
            x,
            y,
            (z+1u)*2,
        );

        let safe_yp = min(y+1, CHUNK_DIM - 1);
        let next_y: u32 = select(chunk.adjacent_blocks[1u][x][z], chunk.blocks[x][safe_yp][z], y < (CHUNK_DIM - 1));
        let y_faces = current ^ next_y;
        let y_dirs = current & (~next_y);
        write_faces_y(
            y_faces,
            y_dirs,
            &index_array,
            &vertex_array,
            &local_count,
            x,
            y,
            (z+1u)*2,
        );

        let current_z_a = current & 0xFFFFu;
        let current_z_b = current >> 16u;
        var z_faces = current_z_a ^ current_z_b;
        var z_dirs = current_z_a & (~current_z_b);

        let adjacent_z = chunk.adjacent_blocks[2u][x][y / 2u];
        let save_zp = min(z+1, CHUNK_DIM - 1);
        let next_z: u32 = select(adjacent_z >> (16 * (y % 2u)), chunk.blocks[x][y][save_zp], z < (CHUNK_DIM_HALF - 1));
        let next_z_a = next_z & 0xFFFFu;
        z_faces |= ((current_z_b ^ next_z_a) << 16u);
        z_dirs |= (current_z_b & (~next_z_a)) << 16u;
        write_faces_z(
            z_faces,
            z_dirs,
            &index_array,
            &vertex_array,
            &local_count,
            x,
            y,
            (z+1u)*2,
        );
    }

    let offset: u32 = atomicAdd(&write_offset, local_count);
    for (var i = 0u; i < (local_count * 4u); i++) {
        vertex_buffer[(offset * 4u) + i] = vertex_array[VOID_OFFSET + i];
    }

    for (var i = 0u; i < (local_count * 6u); i++) {
        index_buffer[(offset * 6u) + i] = (index_array[VOID_OFFSET + i] + (offset * 4u)) - VOID_OFFSET;
    }
}
