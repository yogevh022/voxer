const FACE_ID_BASE_X: u32 = 1u;
const FACE_ID_BASE_Y: u32 = 3u;
const FACE_ID_BASE_Z: u32 = 5u;

fn face_data(
    voxel: u32,
    face_position: vec3<u32>,
    fid: u32,
    ocl_count: vec4<u32>,
    draw_mask: FaceDrawMask,
) -> GPUVoxelFaceData {
    let chunk_y_pos: u32 = bitcast<u32>(wg_chunk_position.y & 0xFF);
    let chunk_z_pos_upper_8bits: u32 = bitcast<u32>(wg_chunk_position.z & 0xFF000);
    
    let word_a = voxel | (fid << 28);
    let word_b = face_position.y | (ocl_count.x << 10) | (ocl_count.y << 30);
    let word_c = face_position.x
        | (face_position.z << 4)
        | (chunk_y_pos << 8)
        | (chunk_z_pos_upper_8bits << 4)
        | (ocl_count.z << 30);
    let word_d = (ocl_count.w << 30);

    return GPUVoxelFaceData(word_a, word_b, word_c, word_d);
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
    fid: u32,
    mask: FaceDrawMask,
    data: GPUVoxelFaceData,
}

fn face_write_args(
    voxel: u32,
    draw_mask: FaceDrawMask,
    fid: u32,
    ocl_count: vec4<u32>,
    face_position: vec3<u32>,
) -> VoxelFaceWriteArgs {
    let face_data = face_data(voxel, face_position, fid, ocl_count, draw_mask);
    return VoxelFaceWriteArgs(fid, draw_mask, face_data);
}

fn x_face_write_args(
    voxel: u32,
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    face_position: vec3<u32>,
) -> VoxelFaceWriteArgs {
    let draw_mask: FaceDrawMask = face_draw_mask((*neighbors)[1][1][1], (*neighbors)[2][1][1]);
    let fid: u32 = FACE_ID_BASE_X - draw_mask.dir;
    let ocl_count = occlusion_count_x(neighbors)[draw_mask.dir];
    return face_write_args(voxel, draw_mask, fid, ocl_count, face_position);
}

fn y_face_write_args(
    voxel: u32,
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    face_position: vec3<u32>,
) -> VoxelFaceWriteArgs {
    let draw_mask: FaceDrawMask = face_draw_mask((*neighbors)[1][1][1], (*neighbors)[1][2][1]);
    let fid: u32 = FACE_ID_BASE_Y - draw_mask.dir;
    let ocl_count = occlusion_count_y(neighbors)[draw_mask.dir];
    return face_write_args(voxel, draw_mask, fid, ocl_count, face_position);
}

fn z_face_write_args(
    voxel: u32,
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    face_position: vec3<u32>,
) -> VoxelFaceWriteArgs {
    let draw_mask = face_draw_mask((*neighbors)[1][1][1], (*neighbors)[1][1][2]);
    let fid: u32 = FACE_ID_BASE_Z - draw_mask.dir;
    let ocl_count = occlusion_count_z(neighbors)[draw_mask.dir];
    return face_write_args(voxel, draw_mask, fid, ocl_count, face_position);
}

fn write_face(face_write_args: VoxelFaceWriteArgs) {
    let fid = face_write_args.fid;
    let draw = face_write_args.mask.draw;
    let fid_data_idx = draw * (pr_face_counts[fid] + VOID_OFFSET);
    pr_face_data[fid][fid_data_idx] = face_write_args.data;
    pr_face_counts[fid] += draw;
}

fn write_xyz_faces(
    voxel: u32,
    neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    face_position: vec3<u32>,
) {
    let x_write_args = x_face_write_args(voxel, neighbors, face_position);
    let y_write_args = y_face_write_args(voxel, neighbors, face_position);
    let z_write_args = z_face_write_args(voxel, neighbors, face_position);
    write_face(x_write_args);
    write_face(y_write_args);
    write_face(z_write_args);
}

fn meshing_pass_at(x: u32, y: u32) {
    var neighbors: array<array<array<u32, 3>, 3>, 3>;
    var face_position = vec3<u32>(x, y, 0);
    var this_voxel: u32;

    for (var z: u32 = 0u; z < CHUNK_DIM; z++) {
        face_position.z = z;
        let offs_x = x + 1;
        let offs_y = y + 1;
        let offs_z = z + 2;
        voxel_neighbors(&neighbors, offs_x, offs_y, offs_z);
        this_voxel = get_u16(wg_chunk_content.blocks[offs_x][offs_y][offs_z / 2], offs_z % 2);
        write_xyz_faces(this_voxel, &neighbors, face_position);
    }

    for (var fid = 0u; fid < 6u; fid++) {
        let offset: u32 = atomicAdd(&wg_face_buffer_write_offsets[fid], pr_face_counts[fid]);
        for (var i = 0u; i < pr_face_counts[fid]; i++) {
            face_data_buffer[offset + i] = pr_face_data[fid][VOID_OFFSET + i];
        }
    }
}

fn opaque_bit(voxel: u32) -> u32 {
    return bit_at(voxel, 15);
}

fn opaque_bit_of_packed(voxel: u32, packed_bit_pos: u32) -> u32 {
    return opaque_bit(get_u16(voxel, packed_bit_pos));
}

fn voxel_neighbors(
    neighbors_out: ptr<function, array<array<array<u32, 3>, 3>, 3>>,
    x: u32,
    y: u32,
    z: u32,
) {
    let xm = x - 1;
    let ym = y - 1;
    let zm = z - 1;
    let xp = x + 1;
    let yp = y + 1;
    let zp = z + 1;

    let half_z = z / 2;
    let half_zm = zm / 2;
    let half_zp = zp / 2;
    let z_bits_pos = z & 1;
    let z_alt_bits_pos = zm & 1;

    (*neighbors_out)[0][0][0] = opaque_bit_of_packed(wg_chunk_content.blocks[xm][ym][half_zm], z_alt_bits_pos);
    (*neighbors_out)[0][0][1] = opaque_bit_of_packed(wg_chunk_content.blocks[xm][ym][half_z], z_bits_pos);
    (*neighbors_out)[0][0][2] = opaque_bit_of_packed(wg_chunk_content.blocks[xm][ym][half_zp], z_alt_bits_pos);

    (*neighbors_out)[0][1][0] = opaque_bit_of_packed(wg_chunk_content.blocks[xm][y][half_zm], z_alt_bits_pos);
    (*neighbors_out)[0][1][1] = opaque_bit_of_packed(wg_chunk_content.blocks[xm][y][half_z], z_bits_pos);
    (*neighbors_out)[0][1][2] = opaque_bit_of_packed(wg_chunk_content.blocks[xm][y][half_zp], z_alt_bits_pos);

    (*neighbors_out)[0][2][0] = opaque_bit_of_packed(wg_chunk_content.blocks[xm][yp][half_zm], z_alt_bits_pos);
    (*neighbors_out)[0][2][1] = opaque_bit_of_packed(wg_chunk_content.blocks[xm][yp][half_z], z_bits_pos);
    (*neighbors_out)[0][2][2] = opaque_bit_of_packed(wg_chunk_content.blocks[xm][yp][half_zp], z_alt_bits_pos);

    (*neighbors_out)[1][0][0] = opaque_bit_of_packed(wg_chunk_content.blocks[x][ym][half_zm], z_alt_bits_pos);
    (*neighbors_out)[1][0][1] = opaque_bit_of_packed(wg_chunk_content.blocks[x][ym][half_z], z_bits_pos);
    (*neighbors_out)[1][0][2] = opaque_bit_of_packed(wg_chunk_content.blocks[x][ym][half_zp], z_alt_bits_pos);

    (*neighbors_out)[1][1][0] = opaque_bit_of_packed(wg_chunk_content.blocks[x][y][half_zm], z_alt_bits_pos);
    (*neighbors_out)[1][1][1] = opaque_bit_of_packed(wg_chunk_content.blocks[x][y][half_z], z_bits_pos);
    (*neighbors_out)[1][1][2] = opaque_bit_of_packed(wg_chunk_content.blocks[x][y][half_zp], z_alt_bits_pos);

    (*neighbors_out)[1][2][0] = opaque_bit_of_packed(wg_chunk_content.blocks[x][yp][half_zm], z_alt_bits_pos);
    (*neighbors_out)[1][2][1] = opaque_bit_of_packed(wg_chunk_content.blocks[x][yp][half_z], z_bits_pos);
    (*neighbors_out)[1][2][2] = opaque_bit_of_packed(wg_chunk_content.blocks[x][yp][half_zp], z_alt_bits_pos);

    (*neighbors_out)[2][0][0] = opaque_bit_of_packed(wg_chunk_content.blocks[xp][ym][half_zm], z_alt_bits_pos);
    (*neighbors_out)[2][0][1] = opaque_bit_of_packed(wg_chunk_content.blocks[xp][ym][half_z], z_bits_pos);
    (*neighbors_out)[2][0][2] = opaque_bit_of_packed(wg_chunk_content.blocks[xp][ym][half_zp], z_alt_bits_pos);

    (*neighbors_out)[2][1][0] = opaque_bit_of_packed(wg_chunk_content.blocks[xp][y][half_zm], z_alt_bits_pos);
    (*neighbors_out)[2][1][1] = opaque_bit_of_packed(wg_chunk_content.blocks[xp][y][half_z], z_bits_pos);
    (*neighbors_out)[2][1][2] = opaque_bit_of_packed(wg_chunk_content.blocks[xp][y][half_zp], z_alt_bits_pos);

    (*neighbors_out)[2][2][0] = opaque_bit_of_packed(wg_chunk_content.blocks[xp][yp][half_zm], z_alt_bits_pos);
    (*neighbors_out)[2][2][1] = opaque_bit_of_packed(wg_chunk_content.blocks[xp][yp][half_z], z_bits_pos);
    (*neighbors_out)[2][2][2] = opaque_bit_of_packed(wg_chunk_content.blocks[xp][yp][half_zp], z_alt_bits_pos);
}
