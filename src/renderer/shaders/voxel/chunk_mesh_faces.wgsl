const FACE_ID_BASE_X: u32 = 0u;
const FACE_ID_BASE_Y: u32 = 3u;
const FACE_ID_BASE_Z: u32 = 5u;

fn pack_face_data(current_voxel: u32, packed_position: u32, fid: u32, illum: u32, ocl_count: u32) -> GPUVoxelFaceData {
    let packed_face_data = (packed_position & 0xFFF)
            | (fid << 12)
            | (illum << 15)
            | (ocl_count << 20);
    let packed_voxel_ypos = (current_voxel << 16) | bitcast<u32>(workgroup_chunk_y_i16_low);
    return GPUVoxelFaceData(packed_face_data, packed_voxel_ypos);
}

fn write_faces_x(
    current_voxel: u32,
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    packed_position: u32,
) {
    let face_draw = current_voxel ^ (*neighbors)[2][1][1];
    let face_dir = current_voxel & (~(*neighbors)[2][1][1]);

    let fid = FACE_ID_BASE_X + face_dir; // + instead of - because x is inversed
    let illum = 0u;
    let ocl_count = occlusion_count_x(neighbors)[face_dir];

    let face_data = pack_face_data(current_voxel, packed_position, fid, illum, ocl_count);

    let private_face_index = face_draw * (private_face_count + FACE_DATA_VOID_OFFSET);
    private_face_data[private_face_index] = face_data;
    private_face_count += 1u * face_draw;
}

fn write_faces_y(
    current_voxel: u32,
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    packed_position: u32,
) {
    let face_draw = current_voxel ^ (*neighbors)[1][2][1];
    let face_dir = current_voxel & (~(*neighbors)[1][2][1]);

    let fid = FACE_ID_BASE_Y - face_dir;
    let illum = 0u;
    let ocl_count = occlusion_count_y(neighbors)[face_dir];

    let face_data = pack_face_data(current_voxel, packed_position, fid, illum, ocl_count);

    let private_face_index = face_draw * (private_face_count + FACE_DATA_VOID_OFFSET);
    private_face_data[private_face_index] = face_data;
    private_face_count += 1u * face_draw;
}

fn write_faces_z(
    current_voxel: u32,
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    packed_position: u32,
) {
    let face_draw = current_voxel ^ (*neighbors)[1][1][2];
    let face_dir = current_voxel & (~(*neighbors)[1][1][2]);

    let fid = FACE_ID_BASE_Z - face_dir;
    let illum = 0u;
    let ocl_count = occlusion_count_z(neighbors)[face_dir];

    let face_data = pack_face_data(current_voxel, packed_position, fid, illum, ocl_count);

    let private_face_index = face_draw * (private_face_count + FACE_DATA_VOID_OFFSET);
    private_face_data[private_face_index] = face_data;
    private_face_count += 1u * face_draw;
}

fn mesh_chunk_position(x: u32, y: u32) {
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

        let current_voxel = bit_at(get_u16(workgroup_chunk_content.blocks[x][y][z / 2u], z % 2), 15);
        let packed_position = (x << 8) | (y << 4) | z;
        write_faces_x(current_voxel, &neighbors, packed_position);
        write_faces_y(current_voxel, &neighbors, packed_position);
        write_faces_z(current_voxel, &neighbors, packed_position);
    }

    let offset: u32 = atomicAdd(&workgroup_buffer_write_offset, private_face_count);
    for (var i = 0u; i < private_face_count; i++) {
        face_data_buffer[offset + i] = private_face_data[FACE_DATA_VOID_OFFSET + i];
    }
}

fn safe_xyz(x: u32, y: u32, z: u32) -> u32 {
    let safe_x_idx = min(x, CHUNK_DIM - 1);
    let safe_y_idx = min(y, CHUNK_DIM - 1);
    let safe_z_idx = min(z, CHUNK_DIM - 1);
    let safe_half_z = safe_z_idx / 2;
    let safe_packed_z_index = safe_z_idx % 2;

    let adj_x = workgroup_chunk_adj_content.next_blocks[0u][safe_y_idx][safe_half_z];
    let adj_y = workgroup_chunk_adj_content.next_blocks[1u][safe_x_idx][safe_half_z];
    let adj_z = get_u16(workgroup_chunk_adj_content.next_blocks[2u][safe_x_idx][safe_y_idx / 2u], safe_y_idx % 2);

    let packed = select(
        select(
            select(
                adj_y,
                adj_x,
                x == CHUNK_DIM
            ),
            adj_z,
            z == CHUNK_DIM
        ),
        workgroup_chunk_content.blocks[safe_x_idx][safe_y_idx][safe_half_z],
        (x != CHUNK_DIM) && (y != CHUNK_DIM) && (z != CHUNK_DIM)
    );
    return get_u16(packed, safe_packed_z_index);
}

fn safe_x(x: u32, y: u32, z: u32) -> u32 {
    let half_z = z / 2u;
    let packed_z_idx = z % 2;

    let safe_x_idx = min(x, CHUNK_DIM - 1);
    let packed = select(
        workgroup_chunk_adj_content.next_blocks[0u][y][half_z],
        workgroup_chunk_content.blocks[safe_x_idx][y][half_z],
        x <= (CHUNK_DIM - 1),
    );
    return get_u16(packed, packed_z_idx);
}

fn safe_y(x: u32, y: u32, z: u32) -> u32 {
    let half_z = z / 2u;
    let packed_z_idx = z % 2;

    let safe_y_idx = min(y, CHUNK_DIM - 1);
    let packed = select(
        workgroup_chunk_adj_content.next_blocks[1u][x][half_z],
        workgroup_chunk_content.blocks[x][safe_y_idx][half_z],
        y <= (CHUNK_DIM - 1),
    );
    return get_u16(packed, packed_z_idx);
}

fn safe_z(x: u32, y: u32, z: u32) -> u32 {
    let safe_pz_idx = min(z, CHUNK_DIM - 1);
    let safe_half_z = safe_pz_idx / 2;
    let safe_packed_z_index = safe_pz_idx % 2;

    let adj_z = get_u16(workgroup_chunk_adj_content.next_blocks[2u][x][y / 2u], y % 2);
    let packed = select(
        adj_z << (16 * safe_packed_z_index),
        workgroup_chunk_content.blocks[x][y][safe_half_z],
        z <= (CHUNK_DIM - 1),
    );
    return get_u16(packed, safe_packed_z_index);
}
