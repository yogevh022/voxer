const FACE_ID_BASE_X: u32 = 1u;
const FACE_ID_BASE_Y: u32 = 3u;
const FACE_ID_BASE_Z: u32 = 5u;

fn write_faces_x(
    current_voxel: u32,
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    x: u32,
    y: u32,
    z: u32,
) {
    let face_draw = current_voxel ^ (*neighbors)[2][1][1];
    let face_dir = current_voxel & (~(*neighbors)[2][1][1]);

    let packed_position = (x * CHUNK_DIM * CHUNK_DIM) + (y * CHUNK_DIM) + z;
    let fid = FACE_ID_BASE_X - face_dir;
    let illum = 0u;
    let ao = vao_x(neighbors)[face_dir];

    // fixme these are low values so unmasked, if for some reason they will exceed exectation this could cause wierdness
    let packed_face_data = (packed_position & 0xFFF)
        | (fid << 12)
        | (illum << 15)
        | (ao << 20);

    let face_data = FaceData(packed_face_data, current_voxel);

    let private_face_index = face_draw * (private_face_count + VOID_OFFSET);
    private_face_data[private_face_index] = face_data;
    private_face_count += 1u * face_draw;
}
//
//fn write_faces_y(
//    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
//    x: f32,
//    y: f32,
//    z: f32,
//) {
//    let uv: vec2<f32> = vec2<f32>(0.5, 0.0);
//
//    let face_draw = (*neighbors)[1][1][1] ^ (*neighbors)[1][2][1];
//    let face_dir = (*neighbors)[1][1][1] & (~(*neighbors)[1][2][1]);
//
//    let v_index = face_draw * ((local_face_count * 4u) + VOID_OFFSET);
//    let i_index = face_draw * ((local_face_count * 6u) + VOID_OFFSET);
//
//    let vertices_ao = vao_y(neighbors)[face_dir];
//
//    quad_indices(face_dir * i_index, v_index);
//    quad_indices_inversed((1u ^ face_dir) * i_index, v_index);
//    quad_vertices_y(vertices_ao, v_index, uv ,x ,y, z);
//    local_face_count += 1u * face_draw;
//}
//
//fn write_faces_z(
//    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
//    x: f32,
//    y: f32,
//    z: f32,
//) {
//    let uv: vec2<f32> = vec2<f32>(0.5, 0.0);
//
//    let face_draw = (*neighbors)[1][1][1] ^ (*neighbors)[1][1][2];
//    let face_dir = (*neighbors)[1][1][1] & (~(*neighbors)[1][1][2]);
//
//    let v_index = face_draw * ((local_face_count * 4u) + VOID_OFFSET);
//    let i_index = face_draw * ((local_face_count * 6u) + VOID_OFFSET);
//
//    var vertices_ao = vao_z(neighbors)[face_dir];
//
//    quad_indices(face_dir * i_index, v_index);
//    quad_indices_inversed((1u ^ face_dir) * i_index, v_index);
//    quad_vertices_z(vertices_ao, v_index, uv ,x ,y, z);
//    local_face_count += 1u * face_draw;
//}

fn mesh_chunk_position(x: u32, y: u32) {
    let x_f32 = f32(x);
    let y_f32 = f32(y);
    for (var z: u32 = 0u; z < CHUNK_DIM; z++) {
        var neighbors: array<array<array<u32, 3>, 3>, 3>;

        neighbors[0][0][0] = bit_at(safe_xyz(x - 1, y - 1, z - 1), 15);
        neighbors[0][0][1] = bit_at(safe_xyz(x - 1, y - 1, z), 15);
        neighbors[0][0][2] = bit_at(safe_xyz(x - 1, y - 1, z + 1), 15);
        neighbors[0][1][0] = bit_at(safe_xyz(x - 1, y, z - 1), 15);
        neighbors[0][1][1] = bit_at(safe_xyz(x - 1, y, z), 15);
        neighbors[0][1][2] = bit_at(safe_xyz(x - 1, y, z + 1), 15);
        neighbors[0][2][0] = bit_at(safe_xyz(x - 1, y + 1, z - 1), 15);
        neighbors[0][2][1] = bit_at(safe_xyz(x - 1, y + 1, z), 15);
        neighbors[0][2][2] = bit_at(safe_xyz(x - 1, y + 1, z + 1), 15);

        neighbors[1][0][0] = bit_at(safe_xyz(x, y - 1, z - 1), 15);
        neighbors[1][0][1] = bit_at(safe_xyz(x, y - 1, z), 15);
        neighbors[1][0][2] = bit_at(safe_xyz(x, y - 1, z + 1), 15);
        neighbors[1][1][0] = bit_at(safe_xyz(x, y, z - 1), 15);
//        neighbors[1][1][1] = current_voxel;
        neighbors[1][1][2] = bit_at(safe_z(x, y, z + 1), 15);
        neighbors[1][2][0] = bit_at(safe_xyz(x, y + 1, z - 1), 15);
        neighbors[1][2][1] = bit_at(safe_y(x, y + 1, z), 15);
        neighbors[1][2][2] = bit_at(safe_xyz(x, y + 1, z + 1), 15);

        neighbors[2][0][0] = bit_at(safe_xyz(x + 1, y - 1, z - 1), 15);
        neighbors[2][0][1] = bit_at(safe_xyz(x + 1, y - 1, z), 15);
        neighbors[2][0][2] = bit_at(safe_xyz(x + 1, y - 1, z + 1), 15);
        neighbors[2][1][0] = bit_at(safe_xyz(x + 1, y, z - 1), 15);
        neighbors[2][1][1] = bit_at(safe_x(x + 1, y, z), 15);
        neighbors[2][1][2] = bit_at(safe_xyz(x + 1, y, z + 1), 15);
        neighbors[2][2][0] = bit_at(safe_xyz(x + 1, y + 1, z - 1), 15);
        neighbors[2][2][1] = bit_at(safe_xyz(x + 1, y + 1, z), 15);
        neighbors[2][2][2] = bit_at(safe_xyz(x + 1, y + 1, z + 1), 15);

        let packed_z_index = z % 2;
        let current_voxel = bit_at(get_u16(workgroup_chunk_blocks[x][y][z / 2u], packed_z_index), 15);

        let z_f32 = f32(z);
        write_faces_x(current_voxel, &neighbors, x_f32, y_f32, z_f32);
//        write_faces_y(&neighbors, x_f32, y_f32, z_f32);
//        write_faces_z(&neighbors, x_f32, y_f32, z_f32);
    }

    let offset: u32 = atomicAdd(&workgroup_buffer_write_offset, private_face_count);
    for (var i = 0u; i < private_face_count; i++) {
        face_data_buffer[offset + i] = private_face_data[VOID_OFFSET + i];
    }
}

fn safe_xyz(x: u32, y: u32, z: u32) -> u32 {
    let safe_x_idx = min(x, CHUNK_DIM - 1);
    let safe_y_idx = min(y, CHUNK_DIM - 1);
    let safe_z_idx = min(z, CHUNK_DIM - 1);
    let safe_half_z = safe_z_idx / 2;
    let safe_packed_z_index = safe_z_idx % 2;

    let adjacent_x = workgroup_chunk_adj_blocks[0u][safe_y_idx][safe_half_z];
    let adjacent_y = workgroup_chunk_adj_blocks[1u][safe_x_idx][safe_half_z];
    let adjacent_z = get_u16(workgroup_chunk_adj_blocks[2u][safe_x_idx][safe_y_idx / 2u], safe_y_idx % 2);

    let packed = select(
        select(
            select(
                adjacent_y,
                adjacent_x,
                x == CHUNK_DIM
            ),
            adjacent_z,
            z == CHUNK_DIM
        ),
        workgroup_chunk_blocks[safe_x_idx][safe_y_idx][safe_half_z],
        (x != CHUNK_DIM) && (y != CHUNK_DIM) && (z != CHUNK_DIM)
    );
    return get_u16(packed, safe_packed_z_index);
}

fn safe_x(x: u32, y: u32, z: u32) -> u32 {
    let half_z = z / 2u;
    let packed_z_idx = z % 2;

    let safe_x_idx = min(x, CHUNK_DIM - 1);
    let packed = select(
        workgroup_chunk_adj_blocks[0u][y][half_z],
        workgroup_chunk_blocks[safe_x_idx][y][half_z],
        x <= (CHUNK_DIM - 1),
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
        y <= (CHUNK_DIM - 1),
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
        z <= (CHUNK_DIM - 1),
    );
    return get_u16(packed, safe_packed_z_index);
}
