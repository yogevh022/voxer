
struct VoxelFace {
    voxel: u32,
    face_id: u32,
}

fn unpack_face_voxel(face_data: GPUVoxelFaceData) -> VoxelFace {
    let voxel = unpack_u16_low(face_data.word_a);
    let face_id = (face_data.word_a >> 28) & 0x7;
    return VoxelFace(voxel, face_id);
}

fn unpack_face_position(face_data: GPUVoxelFaceData) -> vec3<u32> {
    return vec3<u32>(
        face_data.word_c & 0x0F,
        face_data.word_b & 0x0F,
        (face_data.word_c >> 4u) & 0x0F,
    );
}

fn unpack_chunk_position(face_data: GPUVoxelFaceData, packed_xz: u32) -> vec3<i32> {
    let chunk_x: i32 = unpack_i20_low(packed_xz);
    let chunk_y: i32 = unpack_i8_low(face_data.word_c >> 8);
    let chunk_z: i32 = unpack_i20_low(packed_xz >> 20 | (face_data.word_c >> 4) & 0x000FF000);
    return vec3<i32>(
        chunk_x,
        chunk_y,
        chunk_z,
    );
}

fn unpack_face_lighting(face_data: GPUVoxelFaceData) -> array<vec3<u32>, 4> {
    let top_left_lighting = vec3<u32>(
        (face_data.word_a >> 16) & 0x3F,
        (face_data.word_a >> 22) & 0x3F,
        (face_data.word_b >> 4) & 0x3F,
    );

    let top_right_lighting = vec3<u32>(
        (face_data.word_b >> 12) & 0x3F,
        (face_data.word_b >> 18) & 0x3F,
        (face_data.word_b >> 24) & 0x3F,
    );

    let bot_left_lighting = vec3<u32>(
        (face_data.word_c >> 24) & 0x3F,
        (face_data.word_d) & 0x3F,
        (face_data.word_d >> 6) & 0x3F,
    );

    let bot_right_lighting = vec3<u32>(
        (face_data.word_d >> 12) & 0x3F,
        (face_data.word_d >> 18) & 0x3F,
        (face_data.word_d >> 24) & 0x3F,
    );

    return array<vec3<u32>, 4>(
        top_left_lighting,
        top_right_lighting,
        bot_left_lighting,
        bot_right_lighting
    );
}

fn unpack_face_ao(face_data: GPUVoxelFaceData) -> array<u32, 4> {
    let tl_ao = (face_data.word_b >> 10) & 0x03;
    let tr_ao = face_data.word_b >> 30;
    let bl_ao = face_data.word_c >> 30;
    let br_ao = face_data.word_d >> 30;
    return array<u32, 4>(tl_ao, tr_ao, bl_ao, br_ao);
}
