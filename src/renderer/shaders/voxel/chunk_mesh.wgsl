fn face_data(
    voxel: u16,
    face_position: vec3<u32>,
    fid: u32,
) -> GPUVoxelFaceData {
    let word_a = u32(voxel)
        | (face_position.x << 16)
        | (face_position.y << 20)
        | (face_position.z << 24)
        | (fid << 28);
    return GPUVoxelFaceData(word_a);
}

fn write_face(this_position: vec3<u32>, this_voxel: u16, next_voxel: u16, fid_base: u32) {
    let face_draw: u16 = (this_voxel ^ next_voxel) >> 15;
    let face_dir: u32 = u32((this_voxel & (~next_voxel)) >> 15);
    if (face_draw != 0) {
        let fid: u32 = fid_base - face_dir;
        let fdata = face_data(this_voxel, this_position, fid);
        let fid_idx = pr_face_counts[fid];
        pr_face_data[fid][fid_idx] = fdata;
        pr_face_counts[fid] += 1;
    }
}

fn voxel_at(x: u32, y: u32, z: u32) -> u16 {
    return wg_chunk_content.blocks[x][y][z];
}

fn meshing_pass(x: u32, y: u32) {
    const FACE_ID_BASE_X: u32 = 1u;
    const FACE_ID_BASE_Y: u32 = 3u;
    const FACE_ID_BASE_Z: u32 = 5u;

    for (var z: u32 = 0u; z < CHUNK_DIM; z++) {
        let this_voxel: u16 = voxel_at(x + 1, y + 1, z + 1);

        let next_voxel_x: u16 = voxel_at(x + 2, y + 1, z + 1);
        let next_voxel_y: u16 = voxel_at(x + 1, y + 2, z + 1);
        let next_voxel_z: u16 = voxel_at(x + 1, y + 1, z + 2);

        let voxel_position = vec3<u32>(x, y, z);

        write_face(voxel_position, this_voxel, next_voxel_x, FACE_ID_BASE_X);
        write_face(voxel_position, this_voxel, next_voxel_y, FACE_ID_BASE_Y);
        write_face(voxel_position, this_voxel, next_voxel_z, FACE_ID_BASE_Z);
    }

    for (var fid = 0u; fid < 6u; fid++) {
        let offset: u32 = atomicAdd(&wg_face_buffer_write_offsets[fid], pr_face_counts[fid]);
        for (var i = 0u; i < pr_face_counts[fid]; i++) {
            face_data_buffer[offset + i] = pr_face_data[fid][i];
        }
    }
}