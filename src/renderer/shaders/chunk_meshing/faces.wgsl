
fn write_faces_x(
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    x: f32,
    y: f32,
    z: f32,
) {
    let uv: vec2<f32> = vec2<f32>(0.0, 0.0);

    let face_draw = bit_at((*neighbors)[1][1][1] ^ (*neighbors)[2][1][1], 15);
    let face_dir = bit_at((*neighbors)[1][1][1] & (~(*neighbors)[2][1][1]), 15);

    let v_index = face_draw * ((local_face_count * 4u) + VOID_OFFSET);
    let i_index = face_draw * ((local_face_count * 6u) + VOID_OFFSET);

    quad_indices(i_index, v_index);
    minus_x_vertices(face_dir * v_index, uv ,x ,y, z);
    plus_x_vertices((1u ^ face_dir) * v_index, uv, x, y, z);
    local_face_count += 1u * face_draw;
}

fn write_faces_y(
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    x: f32,
    y: f32,
    z: f32,
) {
    let uv: vec2<f32> = vec2<f32>(0.0, 0.0);

    let face_draw = bit_at((*neighbors)[1][1][1] ^ (*neighbors)[1][2][1], 15);
    let face_dir = bit_at((*neighbors)[1][1][1] & (~(*neighbors)[1][2][1]), 15);

    let v_index = face_draw * ((local_face_count * 4u) + VOID_OFFSET);
    let i_index = face_draw * ((local_face_count * 6u) + VOID_OFFSET);

    quad_indices(i_index, v_index);
    plus_y_vertices(face_dir * v_index, uv ,x ,y, z);
    minus_y_vertices((1u ^ face_dir) * v_index, uv, x, y, z);
    local_face_count += 1u * face_draw;
}

fn write_faces_z(
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    x: f32,
    y: f32,
    z: f32,
) {
    let uv: vec2<f32> = vec2<f32>(0.0, 0.0);

    let face_draw = bit_at((*neighbors)[1][1][1] ^ (*neighbors)[1][1][2], 15);
    let face_dir = bit_at((*neighbors)[1][1][1] & (~(*neighbors)[1][1][2]), 15);

    let v_index = face_draw * ((local_face_count * 4u) + VOID_OFFSET);
    let i_index = face_draw * ((local_face_count * 6u) + VOID_OFFSET);

    quad_indices(i_index, v_index);
    plus_z_vertices(face_dir * v_index, uv ,x ,y, z);
    minus_z_vertices((1u ^ face_dir) * v_index, uv, x, y, z);
    local_face_count += 1u * face_draw;
}

fn mesh_chunk_position(x: u32, y: u32) {
    let x_f32 = f32(x);
    let y_f32 = f32(y);
    for (var z: u32 = 0u; z < CHUNK_DIM; z++) {
        var neighbors: array<array<array<u32, 3>, 3>, 3>;
        let packed_z_index = z % 2;
        neighbors[1][1][1] = get_u16(workgroup_chunk_blocks[x][y][z / 2u], packed_z_index);
        neighbors[2][1][1] = safe_x(x+1, y, z);
        neighbors[1][2][1] = safe_y(x, y+1, z);
        neighbors[1][1][2] = safe_z(x, y, z+1);

        let z_f32 = f32(z);
        write_faces_x(&neighbors, x_f32, y_f32, z_f32);
        write_faces_y(&neighbors, x_f32, y_f32, z_f32);
        write_faces_z(&neighbors, x_f32, y_f32, z_f32);
    }

    let offset: u32 = atomicAdd(&workgroup_write_offset, local_face_count);
    for (var i = 0u; i < (local_face_count * 4u); i++) {
        vertex_buffer[(offset * 4u) + i] = local_vertex_array[VOID_OFFSET + i];
    }

    for (var i = 0u; i < (local_face_count * 6u); i++) {
        index_buffer[(offset * 6u) + i] = (local_index_array[VOID_OFFSET + i] + (offset * 4u)) - VOID_OFFSET;
    }
}

fn safe_xyz(x: u32, y: u32, z: u32) -> u32 {
    let safe_px_idx = min(x, CHUNK_DIM - 1);
    let safe_py_idx = min(y, CHUNK_DIM - 1);
    let safe_pz_idx = min(z, CHUNK_DIM - 1);
    return workgroup_chunk_blocks[safe_px_idx][safe_py_idx][safe_pz_idx];
}

fn safe_x(x: u32, y: u32, z: u32) -> u32 {
    let half_z = z / 2u;
    let packed_z_idx = z % 2;

    let safe_x_idx = min(x, CHUNK_DIM - 1);
    let packed = select(
        workgroup_chunk_adj_blocks[0u][y][half_z],
        workgroup_chunk_blocks[safe_x_idx][y][half_z],
        x < (CHUNK_DIM - 1),
    );
    return get_u16(packed, packed_z_idx);
}

fn safe_y(x: u32, y: u32, z: u32) -> u32 {
    let half_z = z / 2u;
    let packed_z_idx = z % 2;

    let safe_y_idx = min(y, CHUNK_DIM - 1);
    let packed = select(
        workgroup_chunk_adj_blocks[1u][x][half_z],
        workgroup_chunk_blocks[x][safe_y_idx][half_z],
        y < (CHUNK_DIM - 1),
    );
    return get_u16(packed, packed_z_idx);
}

fn safe_z(x: u32, y: u32, z: u32) -> u32 {
    let safe_pz_idx = min(z, CHUNK_DIM - 1);
    let safe_half_z = safe_pz_idx / 2;
    let safe_packed_z_index = safe_pz_idx % 2;

    let adjacent_z = get_u16(workgroup_chunk_adj_blocks[2u][x][y / 2u], y % 2);
    let packed = select(
        adjacent_z << (16 * safe_packed_z_index),
        workgroup_chunk_blocks[x][y][safe_half_z],
        z < (CHUNK_DIM - 1),
    );
    return get_u16(packed, safe_packed_z_index);
}
