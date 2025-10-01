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
    let px = min(x + 1, CHUNK_DIM - 1);
    let py = min(y + 1, CHUNK_DIM - 1);
    let mx = max(x, 1) - 1;
    let my = max(y, 1) - 1;

    let x_first = x == 0;
    let x_last = x == CHUNK_DIM - 1;
    let y_first = y == 0;
    let y_last = y == CHUNK_DIM - 1;

    var neighbors = voxel_neighbors_first_z(mx, x, px, x_first, x_last, my, y, py, y_first, y_last, 0);
    var current_voxel = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][0], 0);
    var packed_position = (x << 8) | (y << 4) | 0;
    write_faces_x(current_voxel, &neighbors, packed_position);
    write_faces_y(current_voxel, &neighbors, packed_position);
    write_faces_z(current_voxel, &neighbors, packed_position);

    neighbors = voxel_neighbors_last_z(mx, x, px, x_first, x_last, my, y, py, y_first, y_last, CHUNK_DIM - 1);
    current_voxel = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][(CHUNK_DIM - 1) / 2], 1);
    packed_position = (x << 8) | (y << 4) | (CHUNK_DIM - 1);
    write_faces_x(current_voxel, &neighbors, packed_position);
    write_faces_y(current_voxel, &neighbors, packed_position);
    write_faces_z(current_voxel, &neighbors, packed_position);

    for (var z: u32 = 1u; z < CHUNK_DIM - 1; z++) {
        neighbors = voxel_neighbors_safe_z(mx, x, px, x_first, x_last, my, y, py, y_first, y_last, z);
        current_voxel = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][z / 2u], z % 2);
        packed_position = (x << 8) | (y << 4) | z;
        write_faces_x(current_voxel, &neighbors, packed_position);
        write_faces_y(current_voxel, &neighbors, packed_position);
        write_faces_z(current_voxel, &neighbors, packed_position);
    }

    let offset: u32 = atomicAdd(&workgroup_buffer_write_offset, private_face_count);
    for (var i = 0u; i < private_face_count; i++) {
        face_data_buffer[offset + i] = private_face_data[FACE_DATA_VOID_OFFSET + i];
    }
}

fn px_safe(adj: bool, safe_x: u32, safe_y: u32, half_z: u32) -> u32 {
    return select(
        workgroup_chunk_content.blocks[safe_x][safe_y][half_z],
        workgroup_chunk_adj_content.next_blocks[0u][safe_y][half_z],
        adj,
    );
}

fn py_safe(adj: bool, safe_x: u32, safe_y: u32, half_z: u32) -> u32 {
    return select(
        workgroup_chunk_content.blocks[safe_x][safe_y][half_z],
        workgroup_chunk_adj_content.next_blocks[1u][safe_x][half_z],
        adj,
    );
}

fn mx_safe(adj: bool, safe_x: u32, safe_y: u32, half_z: u32) -> u32 {
    return select(
        workgroup_chunk_content.blocks[safe_x][safe_y][half_z],
        workgroup_chunk_adj_content.prev_blocks[0u][safe_y][half_z],
        adj,
    );
}

fn my_safe(adj: bool, safe_x: u32, safe_y: u32, half_z: u32) -> u32 {
    return select(
        workgroup_chunk_content.blocks[safe_x][safe_y][half_z],
        workgroup_chunk_adj_content.prev_blocks[1u][safe_x][half_z],
        adj,
    );
}

fn pz_safe(safe_x: u32, safe_y: u32) -> u32 {
    return get_u16(workgroup_chunk_adj_content.next_blocks[2u][safe_x][safe_y / 2u], safe_y % 2);
}

fn mz_safe(safe_x: u32, safe_y: u32) -> u32 {
    return get_u16(workgroup_chunk_adj_content.prev_blocks[2u][safe_x][safe_y / 2u], safe_y % 2);
}

fn pxpy_safe(x_adj: bool, y_adj: bool, safe_x: u32, safe_y: u32, half_z: u32) -> u32 {
    return select(px_safe(x_adj, safe_x, safe_y, half_z), py_safe(y_adj, safe_x, safe_y, half_z), x_adj);
}

fn mxmy_safe(x_adj: bool, y_adj: bool, safe_x: u32, safe_y: u32, half_z: u32) -> u32 {
    return select(mx_safe(x_adj, safe_x, safe_y, half_z), my_safe(y_adj, safe_x, safe_y, half_z), x_adj);
}

fn pxmy_safe(x_adj: bool, y_adj: bool, safe_x: u32, safe_y: u32, half_z: u32) -> u32 {
    return select(px_safe(x_adj, safe_x, safe_y, half_z), my_safe(y_adj, safe_x, safe_y, half_z), x_adj);
}

fn mxpy_safe(x_adj: bool, y_adj: bool, safe_x: u32, safe_y: u32, half_z: u32) -> u32 {
    return select(mx_safe(x_adj, safe_x, safe_y, half_z), py_safe(y_adj, safe_x, safe_y, half_z), x_adj);
}

fn opaque_bit_of_packed(voxel: u32, packed_bit_pos: u32) -> u32 {
    return bit_at(get_u16(voxel, packed_bit_pos), 15);
}

fn voxel_neighbors_safe_z(
    mx: u32,
    x: u32,
    px: u32,
    x_first: bool,
    x_last: bool,
    my: u32,
    y: u32,
    py: u32,
    y_first: bool,
    y_last: bool,
    z: u32
) -> array<array<array<u32, 3>, 3>, 3> {
    let half_z = z / 2;
    let half_z_bit_pos = z % 2;
    let mhalf_z = (z - 1) / 2;
    let mhalf_z_bit_pos = mhalf_z % 2;
    let phalf_z = (z + 1) / 2;
    let phalf_z_bit_pos = phalf_z % 2;

    var neighbors: array<array<array<u32, 3>, 3>, 3>;

    neighbors[0][0][0] = opaque_bit_of_packed(mxmy_safe(x_first, y_first, mx, my, mhalf_z), mhalf_z_bit_pos);
    neighbors[0][0][1] = opaque_bit_of_packed(mxmy_safe(x_first, y_first, mx, my, half_z), half_z_bit_pos);
    neighbors[0][0][2] = opaque_bit_of_packed(mxmy_safe(x_first, y_first, mx, my, phalf_z), phalf_z_bit_pos);
    neighbors[0][1][0] = opaque_bit_of_packed(mx_safe(x_first, mx, y, mhalf_z), mhalf_z_bit_pos);
    neighbors[0][1][1] = opaque_bit_of_packed(mx_safe(x_first, mx, y, half_z), half_z_bit_pos);
    neighbors[0][1][2] = opaque_bit_of_packed(mx_safe(x_first, mx, y, phalf_z), phalf_z_bit_pos);
    neighbors[0][2][0] = opaque_bit_of_packed(mxpy_safe(x_first, y_last, mx, py, mhalf_z), mhalf_z_bit_pos);
    neighbors[0][2][1] = opaque_bit_of_packed(mxpy_safe(x_first, y_last, mx, py, half_z), half_z_bit_pos);
    neighbors[0][2][2] = opaque_bit_of_packed(mxpy_safe(x_first, y_last, mx, py, phalf_z), phalf_z_bit_pos);

    neighbors[1][0][0] = opaque_bit_of_packed(my_safe(y_first, x, my, mhalf_z), mhalf_z_bit_pos);
    neighbors[1][0][1] = opaque_bit_of_packed(my_safe(y_first, x, my, half_z), half_z_bit_pos);
    neighbors[1][0][2] = opaque_bit_of_packed(my_safe(y_first, x, my, phalf_z), phalf_z_bit_pos);
    neighbors[1][1][0] = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][mhalf_z], mhalf_z_bit_pos);
    // neighbors[1][1][1] = current_voxel;
    neighbors[1][1][2] = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][phalf_z], phalf_z_bit_pos);
    neighbors[1][2][0] = opaque_bit_of_packed(py_safe(y_last, x, py, mhalf_z), mhalf_z_bit_pos);
    neighbors[1][2][1] = opaque_bit_of_packed(py_safe(y_last, x, py, half_z), half_z_bit_pos);
    neighbors[1][2][2] = opaque_bit_of_packed(py_safe(y_last, x, py, phalf_z), phalf_z_bit_pos);

    neighbors[2][0][0] = opaque_bit_of_packed(pxmy_safe(x_last, y_first, px, my, mhalf_z), mhalf_z_bit_pos);
    neighbors[2][0][1] = opaque_bit_of_packed(pxmy_safe(x_last, y_first, px, my, half_z), half_z_bit_pos);
    neighbors[2][0][2] = opaque_bit_of_packed(pxmy_safe(x_last, y_first, px, my, phalf_z), phalf_z_bit_pos);
    neighbors[2][1][0] = opaque_bit_of_packed(px_safe(x_last, px, y, mhalf_z), mhalf_z_bit_pos);
    neighbors[2][1][1] = opaque_bit_of_packed(px_safe(x_last, px, y, half_z), half_z_bit_pos);
    neighbors[2][1][2] = opaque_bit_of_packed(px_safe(x_last, px, y, phalf_z), phalf_z_bit_pos);
    neighbors[2][2][0] = opaque_bit_of_packed(pxpy_safe(x_last, y_last, px, py, mhalf_z), mhalf_z_bit_pos);
    neighbors[2][2][1] = opaque_bit_of_packed(pxpy_safe(x_last, y_last, px, py, half_z), half_z_bit_pos);
    neighbors[2][2][2] = opaque_bit_of_packed(pxpy_safe(x_last, y_last, px, py, phalf_z), phalf_z_bit_pos);

    return neighbors;
}

fn voxel_neighbors_first_z(
    mx: u32,
    x: u32,
    px: u32,
    x_first: bool,
    x_last: bool,
    my: u32,
    y: u32,
    py: u32,
    y_first: bool,
    y_last: bool,
    z: u32
) -> array<array<array<u32, 3>, 3>, 3> {
    let half_z = z / 2;
    let half_z_bit_pos = z % 2;
    let mhalf_z = (z - 1) / 2;
    let mhalf_z_bit_pos = mhalf_z % 2;
    let phalf_z = (z + 1) / 2;
    let phalf_z_bit_pos = phalf_z % 2;

    var neighbors: array<array<array<u32, 3>, 3>, 3>;

    neighbors[0][0][0] = opaque_bit_of_packed(mz_safe(x, y), mhalf_z_bit_pos);
    neighbors[0][0][1] = opaque_bit_of_packed(mxmy_safe(x_first, y_first, mx, my, half_z), half_z_bit_pos);
    neighbors[0][0][2] = opaque_bit_of_packed(mxmy_safe(x_first, y_first, mx, my, phalf_z), phalf_z_bit_pos);
    neighbors[0][1][0] = opaque_bit_of_packed(mz_safe(x, y), mhalf_z_bit_pos);
    neighbors[0][1][1] = opaque_bit_of_packed(mx_safe(x_first, mx, y, half_z), half_z_bit_pos);
    neighbors[0][1][2] = opaque_bit_of_packed(mx_safe(x_first, mx, y, phalf_z), phalf_z_bit_pos);
    neighbors[0][2][0] = opaque_bit_of_packed(mz_safe(x, y), mhalf_z_bit_pos);
    neighbors[0][2][1] = opaque_bit_of_packed(mxpy_safe(x_first, y_last, mx, py, half_z), half_z_bit_pos);
    neighbors[0][2][2] = opaque_bit_of_packed(mxpy_safe(x_first, y_last, mx, py, phalf_z), phalf_z_bit_pos);

    neighbors[1][0][0] = opaque_bit_of_packed(mz_safe(x, y), mhalf_z_bit_pos);
    neighbors[1][0][1] = opaque_bit_of_packed(my_safe(y_first, x, my, half_z), half_z_bit_pos);
    neighbors[1][0][2] = opaque_bit_of_packed(my_safe(y_first, x, my, phalf_z), phalf_z_bit_pos);
    neighbors[1][1][0] = opaque_bit_of_packed(mz_safe(x, y), mhalf_z_bit_pos);
    // neighbors[1][1][1] = current_voxel;
    neighbors[1][1][2] = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][phalf_z], phalf_z_bit_pos);
    neighbors[1][2][0] = opaque_bit_of_packed(mz_safe(x, y), mhalf_z_bit_pos);
    neighbors[1][2][1] = opaque_bit_of_packed(py_safe(y_last, x, py, half_z), half_z_bit_pos);
    neighbors[1][2][2] = opaque_bit_of_packed(py_safe(y_last, x, py, phalf_z), phalf_z_bit_pos);

    neighbors[2][0][0] = opaque_bit_of_packed(mz_safe(x, y), mhalf_z_bit_pos);
    neighbors[2][0][1] = opaque_bit_of_packed(pxmy_safe(x_last, y_first, px, my, half_z), half_z_bit_pos);
    neighbors[2][0][2] = opaque_bit_of_packed(pxmy_safe(x_last, y_first, px, my, phalf_z), phalf_z_bit_pos);
    neighbors[2][1][0] = opaque_bit_of_packed(mz_safe(x, y), mhalf_z_bit_pos);
    neighbors[2][1][1] = opaque_bit_of_packed(px_safe(x_last, px, y, half_z), half_z_bit_pos);
    neighbors[2][1][2] = opaque_bit_of_packed(px_safe(x_last, px, y, phalf_z), phalf_z_bit_pos);
    neighbors[2][2][0] = opaque_bit_of_packed(mz_safe(x, y), mhalf_z_bit_pos);
    neighbors[2][2][1] = opaque_bit_of_packed(pxpy_safe(x_last, y_last, px, py, half_z), half_z_bit_pos);
    neighbors[2][2][2] = opaque_bit_of_packed(pxpy_safe(x_last, y_last, px, py, phalf_z), phalf_z_bit_pos);

    return neighbors;
}

fn voxel_neighbors_last_z(
    mx: u32,
    x: u32,
    px: u32,
    x_first: bool,
    x_last: bool,
    my: u32,
    y: u32,
    py: u32,
    y_first: bool,
    y_last: bool,
    z: u32
) -> array<array<array<u32, 3>, 3>, 3> {
    let half_z = z / 2;
    let half_z_bit_pos = z % 2;
    let mhalf_z = (z - 1) / 2;
    let mhalf_z_bit_pos = mhalf_z % 2;
    let phalf_z = (z + 1) / 2;
    let phalf_z_bit_pos = phalf_z % 2;

    var neighbors: array<array<array<u32, 3>, 3>, 3>;

    neighbors[0][0][0] = opaque_bit_of_packed(mxmy_safe(x_first, y_first, mx, my, mhalf_z), mhalf_z_bit_pos);
    neighbors[0][0][1] = opaque_bit_of_packed(mxmy_safe(x_first, y_first, mx, my, half_z), half_z_bit_pos);
    neighbors[0][0][2] = opaque_bit_of_packed(pz_safe(x, y), phalf_z_bit_pos);
    neighbors[0][1][0] = opaque_bit_of_packed(mx_safe(x_first, mx, y, mhalf_z), mhalf_z_bit_pos);
    neighbors[0][1][1] = opaque_bit_of_packed(mx_safe(x_first, mx, y, half_z), half_z_bit_pos);
    neighbors[0][1][2] = opaque_bit_of_packed(pz_safe(x, y), phalf_z_bit_pos);
    neighbors[0][2][0] = opaque_bit_of_packed(mxpy_safe(x_first, y_last, mx, py, mhalf_z), mhalf_z_bit_pos);
    neighbors[0][2][1] = opaque_bit_of_packed(mxpy_safe(x_first, y_last, mx, py, half_z), half_z_bit_pos);
    neighbors[0][2][2] = opaque_bit_of_packed(pz_safe(x, y), phalf_z_bit_pos);

    neighbors[1][0][0] = opaque_bit_of_packed(my_safe(y_first, x, my, mhalf_z), mhalf_z_bit_pos);
    neighbors[1][0][1] = opaque_bit_of_packed(my_safe(y_first, x, my, half_z), half_z_bit_pos);
    neighbors[1][0][2] = opaque_bit_of_packed(pz_safe(x, y), phalf_z_bit_pos);
    neighbors[1][1][0] = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][mhalf_z], mhalf_z_bit_pos);
    // neighbors[1][1][1] = current_voxel;
    neighbors[1][1][2] = opaque_bit_of_packed(pz_safe(x, y), phalf_z_bit_pos);
    neighbors[1][2][0] = opaque_bit_of_packed(py_safe(y_last, x, py, mhalf_z), mhalf_z_bit_pos);
    neighbors[1][2][1] = opaque_bit_of_packed(py_safe(y_last, x, py, half_z), half_z_bit_pos);
    neighbors[1][2][2] = opaque_bit_of_packed(pz_safe(x, y), phalf_z_bit_pos);

    neighbors[2][0][0] = opaque_bit_of_packed(pxmy_safe(x_last, y_first, px, my, mhalf_z), mhalf_z_bit_pos);
    neighbors[2][0][1] = opaque_bit_of_packed(pxmy_safe(x_last, y_first, px, my, half_z), half_z_bit_pos);
    neighbors[2][0][2] = opaque_bit_of_packed(pz_safe(x, y), phalf_z_bit_pos);
    neighbors[2][1][0] = opaque_bit_of_packed(px_safe(x_last, px, y, mhalf_z), mhalf_z_bit_pos);
    neighbors[2][1][1] = opaque_bit_of_packed(px_safe(x_last, px, y, half_z), half_z_bit_pos);
    neighbors[2][1][2] = opaque_bit_of_packed(pz_safe(x, y), phalf_z_bit_pos);
    neighbors[2][2][0] = opaque_bit_of_packed(pxpy_safe(x_last, y_last, px, py, mhalf_z), mhalf_z_bit_pos);
    neighbors[2][2][1] = opaque_bit_of_packed(pxpy_safe(x_last, y_last, px, py, half_z), half_z_bit_pos);
    neighbors[2][2][2] = opaque_bit_of_packed(pz_safe(x, y), phalf_z_bit_pos);

    return neighbors;
}