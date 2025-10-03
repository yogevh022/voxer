const FACE_ID_BASE_X: u32 = 0u;
const FACE_ID_BASE_Y: u32 = 3u;
const FACE_ID_BASE_Z: u32 = 5u;

fn pack_xyz_to_12_bits(x: u32, y: u32, z: u32) -> u32 {
    return (x << 8) | (y << 4) | z;
}

fn pack_face_data(current_voxel: u32, packed_local_position: u32, fid: u32, illum: u32, ocl_count: u32) -> GPUVoxelFaceData {
    let packed_face_data = (packed_local_position & 0xFFF)
            | (fid << 12)
            | (illum << 15)
            | (ocl_count << 20);
    let packed_voxel_ypos = (current_voxel << 16) | bitcast<u32>(workgroup_chunk_y_i16_low);
    return GPUVoxelFaceData(packed_face_data, packed_voxel_ypos);
}

struct FaceDrawMask {
    draw: u32,
    dir: u32,
}

fn face_draw_mask(current_voxel: u32, px_voxel: u32) -> FaceDrawMask {
    let face_draw = current_voxel ^ px_voxel;
    let face_dir = current_voxel & (~px_voxel);
    return FaceDrawMask(face_draw, face_dir);
}

fn face_draw_mask_p_only(current_voxel: u32, px_voxel: u32) -> FaceDrawMask {
    let face_draw = current_voxel ^ px_voxel;
    let face_dir = current_voxel & (~px_voxel);
    return FaceDrawMask(face_draw * face_dir, face_dir);
}

fn face_draw_mask_m_only(current_voxel: u32, px_voxel: u32) -> FaceDrawMask {
    let face_draw = current_voxel ^ px_voxel;
    let face_dir = current_voxel & (~px_voxel);
    return FaceDrawMask(face_draw * (1 ^ face_dir), face_dir);
}

fn face_data(
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    packed_local_position: u32,
    fid: u32,
    draw_mask: FaceDrawMask,
) -> GPUVoxelFaceData {
    let illum = 0u;
    let ocl_count = occlusion_count_x(neighbors)[draw_mask.dir];
    let current_voxel = (*neighbors)[1][1][1];
    return pack_face_data(current_voxel, packed_local_position, fid, illum, ocl_count);
}

struct VoxelFaceWriteArgs {
    mask: FaceDrawMask,
    data: GPUVoxelFaceData,
    private_face_idx: u32,
}

fn face_write_args(
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    draw_mask: FaceDrawMask,
    fid: u32,
    packed_local_position: u32,
) -> VoxelFaceWriteArgs {
    let face_data = face_data(neighbors, packed_local_position, fid, draw_mask);
    let private_face_idx = draw_mask.draw * (private_face_count + FACE_DATA_VOID_OFFSET);
    return VoxelFaceWriteArgs(draw_mask, face_data, private_face_idx);
}

fn write_face(face_write_args: VoxelFaceWriteArgs) {
    private_face_data[face_write_args.private_face_idx] = face_write_args.data;
    private_face_count += 1u * face_write_args.mask.draw;
}

fn write_faces_safe_z(
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    packed_local_position: u32,
) {
    var draw_mask: FaceDrawMask;
    var fid: u32;
    var write_args: VoxelFaceWriteArgs;

    draw_mask = face_draw_mask((*neighbors)[1][1][1], (*neighbors)[2][1][1]);
    fid = FACE_ID_BASE_X + draw_mask.dir; // + instead of - because x is inversed
    write_args = face_write_args(neighbors, draw_mask, fid, packed_local_position);
    write_face(write_args);

    draw_mask = face_draw_mask((*neighbors)[1][1][1], (*neighbors)[1][2][1]);
    fid = FACE_ID_BASE_Y - draw_mask.dir;
    write_args = face_write_args(neighbors, draw_mask, fid, packed_local_position);
    write_face(write_args);

    draw_mask = face_draw_mask((*neighbors)[1][1][1], (*neighbors)[1][1][2]);
    fid = FACE_ID_BASE_Z - draw_mask.dir;
    write_args = face_write_args(neighbors, draw_mask, fid, packed_local_position);
    write_face(write_args);
}

fn write_faces_first_z(
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    packed_local_position: u32,
) {
    var draw_mask: FaceDrawMask;
    var fid: u32;
    var write_args: VoxelFaceWriteArgs;

    draw_mask = face_draw_mask_m_only((*neighbors)[1][1][1], (*neighbors)[2][1][1]);
    fid = FACE_ID_BASE_X + draw_mask.dir; // + instead of - because x is inversed
    write_args = face_write_args(neighbors, draw_mask, fid, packed_local_position);
    write_face(write_args);

    draw_mask = face_draw_mask_m_only((*neighbors)[1][1][1], (*neighbors)[1][2][1]);
    fid = FACE_ID_BASE_Y - draw_mask.dir;
    write_args = face_write_args(neighbors, draw_mask, fid, packed_local_position);
    write_face(write_args);

    draw_mask = face_draw_mask_m_only((*neighbors)[1][1][1], (*neighbors)[1][1][2]);
    fid = FACE_ID_BASE_Z - draw_mask.dir;
    write_args = face_write_args(neighbors, draw_mask, fid, packed_local_position);
    write_face(write_args);
}

fn write_faces_last_z(
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    packed_local_position: u32,
) {
    var draw_mask: FaceDrawMask;
    var fid: u32;
    var write_args: VoxelFaceWriteArgs;

    draw_mask = face_draw_mask_p_only((*neighbors)[1][1][1], (*neighbors)[2][1][1]);
    fid = FACE_ID_BASE_X + draw_mask.dir; // + instead of - because x is inversed
    write_args = face_write_args(neighbors, draw_mask, fid, packed_local_position);
    write_face(write_args);

    draw_mask = face_draw_mask_p_only((*neighbors)[1][1][1], (*neighbors)[1][2][1]);
    fid = FACE_ID_BASE_Y - draw_mask.dir;
    write_args = face_write_args(neighbors, draw_mask, fid, packed_local_position);
    write_face(write_args);

    draw_mask = face_draw_mask_p_only((*neighbors)[1][1][1], (*neighbors)[1][1][2]);
    fid = FACE_ID_BASE_Z - draw_mask.dir;
    write_args = face_write_args(neighbors, draw_mask, fid, packed_local_position);
    write_face(write_args);
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

    var neighbors: array<array<array<u32, 3>, 3>, 3>;
    var packed_local_position: u32;

//    let first_z: u32 = 0u;
//    neighbors = voxel_neighbors_first_z(mx, x, px, x_first, x_last, my, y, py, y_first, y_last);
//    packed_local_position = pack_xyz_to_12_bits(x, y, first_z);
//    write_faces_first_z(&neighbors, packed_local_position);
//
//    let last_z: u32 = CHUNK_DIM - 1u;
//    neighbors = voxel_neighbors_last_z(mx, x, px, x_first, x_last, my, y, py, y_first, y_last);
//    packed_local_position = pack_xyz_to_12_bits(x, y, last_z);
//    write_faces_last_z(&neighbors, packed_local_position);

    for (var z: u32 = 1u; z < CHUNK_DIM - 1; z++) {
        neighbors = voxel_neighbors_safe_z(mx, x, px, x_first, x_last, my, y, py, y_first, y_last, z);
        packed_local_position = pack_xyz_to_12_bits(x, y, z);
        write_faces_safe_z(&neighbors, packed_local_position);
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
    neighbors[1][1][1] = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][half_z], half_z_bit_pos);
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
) -> array<array<array<u32, 3>, 3>, 3> {
    let z = 0u;
    let half_z = z / 2;
    let half_z_bit_pos = z % 2;
    let mhalf_z = (z - 1) / 2;
    let mhalf_z_bit_pos = mhalf_z % 2;
    let phalf_z = (z + 1) / 2;
    let phalf_z_bit_pos = phalf_z % 2;

    var neighbors: array<array<array<u32, 3>, 3>, 3>;

    neighbors[0][0][0] = opaque_bit_of_packed(mz_safe(mx, my), 1);
    neighbors[0][0][1] = opaque_bit_of_packed(mxmy_safe(x_first, y_first, mx, my, half_z), half_z_bit_pos);
    neighbors[0][0][2] = opaque_bit_of_packed(mxmy_safe(x_first, y_first, mx, my, phalf_z), phalf_z_bit_pos);
    neighbors[0][1][0] = opaque_bit_of_packed(mz_safe(mx, y), 1);
    neighbors[0][1][1] = opaque_bit_of_packed(mx_safe(x_first, mx, y, half_z), half_z_bit_pos);
    neighbors[0][1][2] = opaque_bit_of_packed(mx_safe(x_first, mx, y, phalf_z), phalf_z_bit_pos);
    neighbors[0][2][0] = opaque_bit_of_packed(mz_safe(mx, py), 1);
    neighbors[0][2][1] = opaque_bit_of_packed(mxpy_safe(x_first, y_last, mx, py, half_z), half_z_bit_pos);
    neighbors[0][2][2] = opaque_bit_of_packed(mxpy_safe(x_first, y_last, mx, py, phalf_z), phalf_z_bit_pos);

    neighbors[1][0][0] = opaque_bit_of_packed(mz_safe(x, my), 1);
    neighbors[1][0][1] = opaque_bit_of_packed(my_safe(y_first, x, my, half_z), half_z_bit_pos);
    neighbors[1][0][2] = opaque_bit_of_packed(my_safe(y_first, x, my, phalf_z), phalf_z_bit_pos);
    neighbors[1][1][0] = opaque_bit_of_packed(mz_safe(x, y), 1);
    neighbors[1][1][1] = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][half_z], half_z_bit_pos);
    neighbors[1][1][2] = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][phalf_z], phalf_z_bit_pos);
    neighbors[1][2][0] = opaque_bit_of_packed(mz_safe(x, py), 1);
    neighbors[1][2][1] = opaque_bit_of_packed(py_safe(y_last, x, py, half_z), half_z_bit_pos);
    neighbors[1][2][2] = opaque_bit_of_packed(py_safe(y_last, x, py, phalf_z), phalf_z_bit_pos);

    neighbors[2][0][0] = opaque_bit_of_packed(mz_safe(px, my), 1);
    neighbors[2][0][1] = opaque_bit_of_packed(pxmy_safe(x_last, y_first, px, my, half_z), half_z_bit_pos);
    neighbors[2][0][2] = opaque_bit_of_packed(pxmy_safe(x_last, y_first, px, my, phalf_z), phalf_z_bit_pos);
    neighbors[2][1][0] = opaque_bit_of_packed(mz_safe(px, y), 1);
    neighbors[2][1][1] = opaque_bit_of_packed(px_safe(x_last, px, y, half_z), half_z_bit_pos);
    neighbors[2][1][2] = opaque_bit_of_packed(px_safe(x_last, px, y, phalf_z), phalf_z_bit_pos);
    neighbors[2][2][0] = opaque_bit_of_packed(mz_safe(px, py), 1);
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
) -> array<array<array<u32, 3>, 3>, 3> {
    let z = CHUNK_DIM - 1;
    let half_z = z / 2;
    let half_z_bit_pos = z % 2;
    let mhalf_z = (z - 1) / 2;
    let mhalf_z_bit_pos = mhalf_z % 2;
    let phalf_z = (z + 1) / 2;
    let phalf_z_bit_pos = phalf_z % 2;

    var neighbors: array<array<array<u32, 3>, 3>, 3>;

    neighbors[0][0][0] = opaque_bit_of_packed(mxmy_safe(x_first, y_first, mx, my, mhalf_z), mhalf_z_bit_pos);
    neighbors[0][0][1] = opaque_bit_of_packed(mxmy_safe(x_first, y_first, mx, my, half_z), half_z_bit_pos);
    neighbors[0][0][2] = opaque_bit_of_packed(pz_safe(mx, my), 0);
    neighbors[0][1][0] = opaque_bit_of_packed(mx_safe(x_first, mx, y, mhalf_z), mhalf_z_bit_pos);
    neighbors[0][1][1] = opaque_bit_of_packed(mx_safe(x_first, mx, y, half_z), half_z_bit_pos);
    neighbors[0][1][2] = opaque_bit_of_packed(pz_safe(mx, y), 0);
    neighbors[0][2][0] = opaque_bit_of_packed(mxpy_safe(x_first, y_last, mx, py, mhalf_z), mhalf_z_bit_pos);
    neighbors[0][2][1] = opaque_bit_of_packed(mxpy_safe(x_first, y_last, mx, py, half_z), half_z_bit_pos);
    neighbors[0][2][2] = opaque_bit_of_packed(pz_safe(mx, py), 0);

    neighbors[1][0][0] = opaque_bit_of_packed(my_safe(y_first, x, my, mhalf_z), mhalf_z_bit_pos);
    neighbors[1][0][1] = opaque_bit_of_packed(my_safe(y_first, x, my, half_z), half_z_bit_pos);
    neighbors[1][0][2] = opaque_bit_of_packed(pz_safe(x, my), 0);
    neighbors[1][1][0] = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][mhalf_z], mhalf_z_bit_pos);
    neighbors[1][1][1] = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][half_z], half_z_bit_pos);
    neighbors[1][1][2] = opaque_bit_of_packed(pz_safe(x, y), 0);
    neighbors[1][2][0] = opaque_bit_of_packed(py_safe(y_last, x, py, mhalf_z), mhalf_z_bit_pos);
    neighbors[1][2][1] = opaque_bit_of_packed(py_safe(y_last, x, py, half_z), half_z_bit_pos);
    neighbors[1][2][2] = opaque_bit_of_packed(pz_safe(x, py), 0);

    neighbors[2][0][0] = opaque_bit_of_packed(pxmy_safe(x_last, y_first, px, my, mhalf_z), mhalf_z_bit_pos);
    neighbors[2][0][1] = opaque_bit_of_packed(pxmy_safe(x_last, y_first, px, my, half_z), half_z_bit_pos);
    neighbors[2][0][2] = opaque_bit_of_packed(pz_safe(px, my), 0);
    neighbors[2][1][0] = opaque_bit_of_packed(px_safe(x_last, px, y, mhalf_z), mhalf_z_bit_pos);
    neighbors[2][1][1] = opaque_bit_of_packed(px_safe(x_last, px, y, half_z), half_z_bit_pos);
    neighbors[2][1][2] = opaque_bit_of_packed(pz_safe(px, y), 0);
    neighbors[2][2][0] = opaque_bit_of_packed(pxpy_safe(x_last, y_last, px, py, mhalf_z), mhalf_z_bit_pos);
    neighbors[2][2][1] = opaque_bit_of_packed(pxpy_safe(x_last, y_last, px, py, half_z), half_z_bit_pos);
    neighbors[2][2][2] = opaque_bit_of_packed(pz_safe(px, py), 0);

    return neighbors;
}
//
//
//fn write_faces_y(
//    current_voxel: u32,
//    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
//    packed_position: u32,
//) {
//    let face_draw = current_voxel ^ (*neighbors)[1][2][1];
//    let face_dir = current_voxel & (~(*neighbors)[1][2][1]);
//
//    let fid = FACE_ID_BASE_Y - face_dir;
//    let illum = 0u;
//    let ocl_count = occlusion_count_y(neighbors)[face_dir];
//
//    let face_data = pack_face_data(current_voxel, packed_position, fid, illum, ocl_count);
//
//    let private_face_index = face_draw * (private_face_count + FACE_DATA_VOID_OFFSET);
//    private_face_data[private_face_index] = face_data;
//    private_face_count += 1u * face_draw;
//}
//
//fn write_faces_z(
//    current_voxel: u32,
//    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
//    packed_position: u32,
//) {
//    let face_draw = current_voxel ^ (*neighbors)[1][1][2];
//    let face_dir = current_voxel & (~(*neighbors)[1][1][2]);
//
//    let fid = FACE_ID_BASE_Z - face_dir;
//    let illum = 0u;
//    let ocl_count = occlusion_count_z(neighbors)[face_dir];
//
//    let face_data = pack_face_data(current_voxel, packed_position, fid, illum, ocl_count);
//
//    let private_face_index = face_draw * (private_face_count + FACE_DATA_VOID_OFFSET);
//    private_face_data[private_face_index] = face_data;
//    private_face_count += 1u * face_draw;
//}

//    neighbors = voxel_neighbors_first_z(mx, x, px, x_first, x_last, my, y, py, y_first, y_last, 0);
//    current_voxel = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][0], 0);
//    packed_position = (x << 8) | (y << 4) | 0;
//    face_x_draw_data(current_voxel, &neighbors, packed_position);
//    write_faces_y(current_voxel, &neighbors, packed_position);
//    write_faces_z(current_voxel, &neighbors, packed_position);
//
//    neighbors = voxel_neighbors_last_z(mx, x, px, x_first, x_last, my, y, py, y_first, y_last, CHUNK_DIM - 1);
//    current_voxel = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][(CHUNK_DIM - 1) / 2], 1);
//    packed_position = (x << 8) | (y << 4) | (CHUNK_DIM - 1);
//    face_x_draw_data(current_voxel, &neighbors, packed_position);
//    write_faces_y(current_voxel, &neighbors, packed_position);
//    write_faces_z(current_voxel, &neighbors, packed_position);