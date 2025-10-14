const FACE_ID_BASE_X: u32 = 0u;
const FACE_ID_BASE_Y: u32 = 3u;
const FACE_ID_BASE_Z: u32 = 5u;

fn face_data(
    current_voxel: u32,
    face_position: vec3<u32>,
    fid: u32,
    ocl_count: vec4<u32>,
    draw_mask: FaceDrawMask,
) -> GPUVoxelFaceData {
    let global_position = workgroup_chunk_world_position + vec3<i32>(face_position);

    let word_a = cast_i24_to_u32(global_position.x) | (ocl_count.x << 30);
    let word_b = cast_i24_to_u32(global_position.z) | (ocl_count.y << 30);
    let word_c = cast_i12_to_u32(global_position.y) | (ocl_count.w << 30);
    let word_d = (ocl_count.z << 30);
    let word_e = current_voxel | (fid << 28);

    return GPUVoxelFaceData(word_a, word_b, word_c, word_d, word_e);
}

struct FaceDrawMask {
    draw: u32,
    dir: u32,
}

fn face_draw_mask(current_voxel: u32, next_voxel: u32) -> FaceDrawMask {
    let face_draw = current_voxel ^ next_voxel;
    let face_dir = current_voxel & (~next_voxel);
    return FaceDrawMask(face_draw, face_dir);
}

struct VoxelFaceWriteArgs {
    mask: FaceDrawMask,
    data: GPUVoxelFaceData,
    private_face_idx: u32,
}

fn face_write_args(
    current_voxel: u32,
    draw_mask: FaceDrawMask,
    fid: u32,
    ocl_count: vec4<u32>,
    face_position: vec3<u32>,
) -> VoxelFaceWriteArgs {
    let face_data = face_data(current_voxel, face_position, fid, ocl_count, draw_mask);
    let private_face_idx = draw_mask.draw * (private_face_count + FACE_DATA_VOID_OFFSET);
    return VoxelFaceWriteArgs(draw_mask, face_data, private_face_idx);
}

fn write_face(face_write_args: VoxelFaceWriteArgs) {
    private_face_data[face_write_args.private_face_idx] = face_write_args.data;
    private_face_count += face_write_args.mask.draw;
}

fn write_x_face(
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    face_position: vec3<u32>,
) {
    let draw_mask: FaceDrawMask = face_draw_mask((*neighbors)[1][1][1], (*neighbors)[2][1][1]);
    let fid: u32 = FACE_ID_BASE_X + draw_mask.dir; // + instead of - because x is inversed
    let ocl_count = occlusion_count_x(neighbors)[draw_mask.dir];
    // fixme current voxel is wrong, neighbors contains opaque bit not entire voxel
    let current_voxel = (*neighbors)[1][1][1];
    let write_args: VoxelFaceWriteArgs = face_write_args(current_voxel, draw_mask, fid, ocl_count, face_position);
    write_face(write_args);
}

fn write_y_face(
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    face_position: vec3<u32>,
) {
    let draw_mask: FaceDrawMask = face_draw_mask((*neighbors)[1][1][1], (*neighbors)[1][2][1]);
    let fid: u32 = FACE_ID_BASE_Y - draw_mask.dir;
    let ocl_count = occlusion_count_y(neighbors)[draw_mask.dir];
    let current_voxel = (*neighbors)[1][1][1];
    let write_args: VoxelFaceWriteArgs = face_write_args(current_voxel, draw_mask, fid, ocl_count, face_position);
    write_face(write_args);
}

fn write_z_face(
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    face_position: vec3<u32>,
) {
    let draw_mask = face_draw_mask((*neighbors)[1][1][1], (*neighbors)[1][1][2]);
    let fid = FACE_ID_BASE_Z - draw_mask.dir;
    let ocl_count = occlusion_count_z(neighbors)[draw_mask.dir];
    let current_voxel = (*neighbors)[1][1][1];
    let write_args = face_write_args(current_voxel, draw_mask, fid, ocl_count, face_position);
    write_face(write_args);
}

fn write_xyz_faces(
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    face_position: vec3<u32>,
) {
    write_x_face(neighbors, face_position);
    write_y_face(neighbors, face_position);
    write_z_face(neighbors, face_position);
}

fn meshing_pass_at(x: u32, y: u32) {
    let px = min(x + 1, CHUNK_DIM - 1);
    let py = min(y + 1, CHUNK_DIM - 1);
    let mx = max(x, 1) - 1;
    let my = max(y, 1) - 1;
    var neighbors: array<array<array<u32, 3>, 3>, 3>;

    var first_voxel = vec3<bool>(x == 0, y == 0, false);
    var last_voxel = vec3<bool>(x == CHUNK_DIM - 1, y == CHUNK_DIM - 1, false);
    var face_position = vec3<u32>(x, y, 0);

    // xy at (z != first/last)
    for (var z: u32 = 1u; z < CHUNK_DIM - 1; z++) {
        face_position.z = z;
        let mz = z - 1;
        let pz = z + 1;
        neighbors = voxel_neighbors_safe_z(first_voxel, last_voxel, mx, x, px, my, y, py, mz, z, pz);
        write_xyz_faces(&neighbors, face_position);
    }

    // xy at first z
    face_position.z = 0;
    first_voxel.z = true;
    neighbors = voxel_neighbors_first_z(first_voxel, last_voxel, mx, x, px, my, y, py);
    write_xyz_faces(&neighbors, face_position);

    // xy at last z
    face_position.z = CHUNK_DIM - 1;
    first_voxel.z = false;
    last_voxel.z = true;
    neighbors = voxel_neighbors_last_z(first_voxel, last_voxel, mx, x, px, my, y, py);
    write_xyz_faces(&neighbors, face_position);

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

fn mx_safe(adj: bool, safe_x: u32, safe_y: u32, half_z: u32) -> u32 {
    return select(
        workgroup_chunk_content.blocks[safe_x][safe_y][half_z],
        workgroup_chunk_adj_content.prev_blocks[0u][safe_y][half_z],
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
    return select(py_safe(y_adj, safe_x, safe_y, half_z), px_safe(x_adj, safe_x, safe_y, half_z), x_adj);
}

fn pxmy_safe(x_adj: bool, y_adj: bool, safe_x: u32, safe_y: u32, half_z: u32) -> u32 {
    return select(my_safe(y_adj, safe_x, safe_y, half_z), px_safe(x_adj, safe_x, safe_y, half_z), x_adj);
}

fn mxpy_safe(x_adj: bool, y_adj: bool, safe_x: u32, safe_y: u32, half_z: u32) -> u32 {
    return select(py_safe(y_adj, safe_x, safe_y, half_z), mx_safe(x_adj, safe_x, safe_y, half_z), x_adj);
}

fn mxmy_safe(x_adj: bool, y_adj: bool, safe_x: u32, safe_y: u32, half_z: u32) -> u32 {
    return select(my_safe(y_adj, safe_x, safe_y, half_z), mx_safe(x_adj, safe_x, safe_y, half_z), x_adj);
}

fn opaque_bit(voxel: u32) -> u32 {
    return bit_at(voxel, 15);
}

fn opaque_bit_of_packed(voxel: u32, packed_bit_pos: u32) -> u32 {
    return opaque_bit(get_u16(voxel, packed_bit_pos));
}

fn voxel_neighbors_safe_z(
    first_voxel: vec3<bool>,
    last_voxel: vec3<bool>,
    mx: u32,
    x: u32,
    px: u32,
    my: u32,
    y: u32,
    py: u32,
    mz: u32,
    z: u32,
    pz: u32,
) -> array<array<array<u32, 3>, 3>, 3> {
    let half_z = z / 2;
    let half_z_bit_pos = z % 2;
    let mhalf_z = mz / 2;
    let mhalf_z_bit_pos = mz % 2;
    let phalf_z = pz / 2;
    let phalf_z_bit_pos = pz % 2;
    var neighbors: array<array<array<u32, 3>, 3>, 3>;

    neighbors[0][0][0] = opaque_bit_of_packed(mxmy_safe(first_voxel.x, first_voxel.y, mx, my, mhalf_z), mhalf_z_bit_pos);
    neighbors[0][0][1] = opaque_bit_of_packed(mxmy_safe(first_voxel.x, first_voxel.y, mx, my, half_z), half_z_bit_pos);
    neighbors[0][0][2] = opaque_bit_of_packed(mxmy_safe(first_voxel.x, first_voxel.y, mx, my, phalf_z), phalf_z_bit_pos);
    neighbors[0][1][0] = opaque_bit_of_packed(mx_safe(first_voxel.x, mx, y, mhalf_z), mhalf_z_bit_pos);
    neighbors[0][1][1] = opaque_bit_of_packed(mx_safe(first_voxel.x, mx, y, half_z), half_z_bit_pos);
    neighbors[0][1][2] = opaque_bit_of_packed(mx_safe(first_voxel.x, mx, y, phalf_z), phalf_z_bit_pos);
    neighbors[0][2][0] = opaque_bit_of_packed(mxpy_safe(first_voxel.x, last_voxel.y, mx, py, mhalf_z), mhalf_z_bit_pos);
    neighbors[0][2][1] = opaque_bit_of_packed(mxpy_safe(first_voxel.x, last_voxel.y, mx, py, half_z), half_z_bit_pos);
    neighbors[0][2][2] = opaque_bit_of_packed(mxpy_safe(first_voxel.x, last_voxel.y, mx, py, phalf_z), phalf_z_bit_pos);

    neighbors[1][0][0] = opaque_bit_of_packed(my_safe(first_voxel.y, x, my, mhalf_z), mhalf_z_bit_pos);
    neighbors[1][0][1] = opaque_bit_of_packed(my_safe(first_voxel.y, x, my, half_z), half_z_bit_pos);
    neighbors[1][0][2] = opaque_bit_of_packed(my_safe(first_voxel.y, x, my, phalf_z), phalf_z_bit_pos);
    neighbors[1][1][0] = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][mhalf_z], mhalf_z_bit_pos);
    neighbors[1][1][1] = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][half_z], half_z_bit_pos);
    neighbors[1][1][2] = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][phalf_z], phalf_z_bit_pos);
    neighbors[1][2][0] = opaque_bit_of_packed(py_safe(last_voxel.y, x, py, mhalf_z), mhalf_z_bit_pos);
    neighbors[1][2][1] = opaque_bit_of_packed(py_safe(last_voxel.y, x, py, half_z), half_z_bit_pos);
    neighbors[1][2][2] = opaque_bit_of_packed(py_safe(last_voxel.y, x, py, phalf_z), phalf_z_bit_pos);

    neighbors[2][0][0] = opaque_bit_of_packed(pxmy_safe(last_voxel.x, first_voxel.y, px, my, mhalf_z), mhalf_z_bit_pos);
    neighbors[2][0][1] = opaque_bit_of_packed(pxmy_safe(last_voxel.x, first_voxel.y, px, my, half_z), half_z_bit_pos);
    neighbors[2][0][2] = opaque_bit_of_packed(pxmy_safe(last_voxel.x, first_voxel.y, px, my, phalf_z), phalf_z_bit_pos);
    neighbors[2][1][0] = opaque_bit_of_packed(px_safe(last_voxel.x, px, y, mhalf_z), mhalf_z_bit_pos);
    neighbors[2][1][1] = opaque_bit_of_packed(px_safe(last_voxel.x, px, y, half_z), half_z_bit_pos);
    neighbors[2][1][2] = opaque_bit_of_packed(px_safe(last_voxel.x, px, y, phalf_z), phalf_z_bit_pos);
    neighbors[2][2][0] = opaque_bit_of_packed(pxpy_safe(last_voxel.x, last_voxel.y, px, py, mhalf_z), mhalf_z_bit_pos);
    neighbors[2][2][1] = opaque_bit_of_packed(pxpy_safe(last_voxel.x, last_voxel.y, px, py, half_z), half_z_bit_pos);
    neighbors[2][2][2] = opaque_bit_of_packed(pxpy_safe(last_voxel.x, last_voxel.y, px, py, phalf_z), phalf_z_bit_pos);

    return neighbors;
}

fn voxel_neighbors_first_z(
    first_voxel: vec3<bool>,
    last_voxel: vec3<bool>,
    mx: u32,
    x: u32,
    px: u32,
    my: u32,
    y: u32,
    py: u32,
) -> array<array<array<u32, 3>, 3>, 3> {
    let half_z = 0u;
    let half_z_bit_pos = 0u;
    let phalf_z = 0u;
    let phalf_z_bit_pos = 1u;
    var neighbors: array<array<array<u32, 3>, 3>, 3>;

    neighbors[0][0][0] = opaque_bit(mz_safe(mx, my));
    neighbors[0][0][1] = opaque_bit_of_packed(mxmy_safe(first_voxel.x, first_voxel.y, mx, my, half_z), half_z_bit_pos);
    neighbors[0][0][2] = opaque_bit_of_packed(mxmy_safe(first_voxel.x, first_voxel.y, mx, my, phalf_z), phalf_z_bit_pos);
    neighbors[0][1][0] = opaque_bit(mz_safe(mx, y));
    neighbors[0][1][1] = opaque_bit_of_packed(mx_safe(first_voxel.x, mx, y, half_z), half_z_bit_pos);
    neighbors[0][1][2] = opaque_bit_of_packed(mx_safe(first_voxel.x, mx, y, phalf_z), phalf_z_bit_pos);
    neighbors[0][2][0] = opaque_bit(mz_safe(mx, py));
    neighbors[0][2][1] = opaque_bit_of_packed(mxpy_safe(first_voxel.x, last_voxel.y, mx, py, half_z), half_z_bit_pos);
    neighbors[0][2][2] = opaque_bit_of_packed(mxpy_safe(first_voxel.x, last_voxel.y, mx, py, phalf_z), phalf_z_bit_pos);

    neighbors[1][0][0] = opaque_bit(mz_safe(x, my));
    neighbors[1][0][1] = opaque_bit_of_packed(my_safe(first_voxel.y, x, my, half_z), half_z_bit_pos);
    neighbors[1][0][2] = opaque_bit_of_packed(my_safe(first_voxel.y, x, my, phalf_z), phalf_z_bit_pos);
    neighbors[1][1][0] = opaque_bit(mz_safe(x, y));
    neighbors[1][1][1] = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][half_z], half_z_bit_pos);
    neighbors[1][1][2] = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][phalf_z], phalf_z_bit_pos);
    neighbors[1][2][0] = opaque_bit(mz_safe(x, py));
    neighbors[1][2][1] = opaque_bit_of_packed(py_safe(last_voxel.y, x, py, half_z), half_z_bit_pos);
    neighbors[1][2][2] = opaque_bit_of_packed(py_safe(last_voxel.y, x, py, phalf_z), phalf_z_bit_pos);

    neighbors[2][0][0] = opaque_bit(mz_safe(px, my));
    neighbors[2][0][1] = opaque_bit_of_packed(pxmy_safe(last_voxel.x, first_voxel.y, px, my, half_z), half_z_bit_pos);
    neighbors[2][0][2] = opaque_bit_of_packed(pxmy_safe(last_voxel.x, first_voxel.y, px, my, phalf_z), phalf_z_bit_pos);
    neighbors[2][1][0] = opaque_bit(mz_safe(px, y));
    neighbors[2][1][1] = opaque_bit_of_packed(px_safe(last_voxel.x, px, y, half_z), half_z_bit_pos);
    neighbors[2][1][2] = opaque_bit_of_packed(px_safe(last_voxel.x, px, y, phalf_z), phalf_z_bit_pos);
    neighbors[2][2][0] = opaque_bit(mz_safe(px, py));
    neighbors[2][2][1] = opaque_bit_of_packed(pxpy_safe(last_voxel.x, last_voxel.y, px, py, half_z), half_z_bit_pos);
    neighbors[2][2][2] = opaque_bit_of_packed(pxpy_safe(last_voxel.x, last_voxel.y, px, py, phalf_z), phalf_z_bit_pos);

    return neighbors;
}

fn voxel_neighbors_last_z(
    first_voxel: vec3<bool>,
    last_voxel: vec3<bool>,
    mx: u32,
    x: u32,
    px: u32,
    my: u32,
    y: u32,
    py: u32,
) -> array<array<array<u32, 3>, 3>, 3> {
    let z = CHUNK_DIM - 1;
    let half_z = CHUNK_DIM_HALF - 1;
    let half_z_bit_pos = 1u;
    let mhalf_z = CHUNK_DIM_HALF - 1;
    let mhalf_z_bit_pos = 0u;
    var neighbors: array<array<array<u32, 3>, 3>, 3>;

    neighbors[0][0][0] = opaque_bit_of_packed(mxmy_safe(first_voxel.x, first_voxel.y, mx, my, mhalf_z), mhalf_z_bit_pos);
    neighbors[0][0][1] = opaque_bit_of_packed(mxmy_safe(first_voxel.x, first_voxel.y, mx, my, half_z), half_z_bit_pos);
    neighbors[0][0][2] = opaque_bit(pz_safe(mx, my));
    neighbors[0][1][0] = opaque_bit_of_packed(mx_safe(first_voxel.x, mx, y, mhalf_z), mhalf_z_bit_pos);
    neighbors[0][1][1] = opaque_bit_of_packed(mx_safe(first_voxel.x, mx, y, half_z), half_z_bit_pos);
    neighbors[0][1][2] = opaque_bit(pz_safe(mx, y));
    neighbors[0][2][0] = opaque_bit_of_packed(mxpy_safe(first_voxel.x, last_voxel.y, mx, py, mhalf_z), mhalf_z_bit_pos);
    neighbors[0][2][1] = opaque_bit_of_packed(mxpy_safe(first_voxel.x, last_voxel.y, mx, py, half_z), half_z_bit_pos);
    neighbors[0][2][2] = opaque_bit(pz_safe(mx, py));

    neighbors[1][0][0] = opaque_bit_of_packed(my_safe(first_voxel.y, x, my, mhalf_z), mhalf_z_bit_pos);
    neighbors[1][0][1] = opaque_bit_of_packed(my_safe(first_voxel.y, x, my, half_z), half_z_bit_pos);
    neighbors[1][0][2] = opaque_bit(pz_safe(x, my));
    neighbors[1][1][0] = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][mhalf_z], mhalf_z_bit_pos);
    neighbors[1][1][1] = opaque_bit_of_packed(workgroup_chunk_content.blocks[x][y][half_z], half_z_bit_pos);
    neighbors[1][1][2] = opaque_bit(pz_safe(x, y));
    neighbors[1][2][0] = opaque_bit_of_packed(py_safe(last_voxel.y, x, py, mhalf_z), mhalf_z_bit_pos);
    neighbors[1][2][1] = opaque_bit_of_packed(py_safe(last_voxel.y, x, py, half_z), half_z_bit_pos);
    neighbors[1][2][2] = opaque_bit(pz_safe(x, py));

    neighbors[2][0][0] = opaque_bit_of_packed(pxmy_safe(last_voxel.x, first_voxel.y, px, my, mhalf_z), mhalf_z_bit_pos);
    neighbors[2][0][1] = opaque_bit_of_packed(pxmy_safe(last_voxel.x, first_voxel.y, px, my, half_z), half_z_bit_pos);
    neighbors[2][0][2] = opaque_bit(pz_safe(px, my));
    neighbors[2][1][0] = opaque_bit_of_packed(px_safe(last_voxel.x, px, y, mhalf_z), mhalf_z_bit_pos);
    neighbors[2][1][1] = opaque_bit_of_packed(px_safe(last_voxel.x, px, y, half_z), half_z_bit_pos);
    neighbors[2][1][2] = opaque_bit(pz_safe(px, y));
    neighbors[2][2][0] = opaque_bit_of_packed(pxpy_safe(last_voxel.x, last_voxel.y, px, py, mhalf_z), mhalf_z_bit_pos);
    neighbors[2][2][1] = opaque_bit_of_packed(pxpy_safe(last_voxel.x, last_voxel.y, px, py, half_z), half_z_bit_pos);
    neighbors[2][2][2] = opaque_bit(pz_safe(px, py));

    return neighbors;
}