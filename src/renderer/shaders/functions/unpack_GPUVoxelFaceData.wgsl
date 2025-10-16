
struct VoxelFace {
    voxel: u32,
    face_id: u32,
}

fn unpack_face_voxel(face_data: GPUVoxelFaceData) -> VoxelFace {
    let voxel = unpack_u16_low(face_data.word_e);
    let face_id = (face_data.word_e >> 28) & 0x7;
    return VoxelFace(voxel, face_id);
}

fn unpack_face_position(face_data: GPUVoxelFaceData) -> vec3<i32> {
    return vec3<i32>(
        unpack_i24_low(face_data.word_a),
        unpack_i12_low(face_data.word_c),
        unpack_i24_low(face_data.word_b),
    );
}

fn unpack_face_lighting(face_data: GPUVoxelFaceData) -> array<vec3<u32>, 4> {
    let top_left_lighting = vec3<u32>(
        (face_data.word_a >> 24) & 0x3F,
        (face_data.word_d >> 18) & 0x3F,
        (face_data.word_d >> 24) & 0x3F,
    );

    let top_right_lighting = vec3<u32>(
        (face_data.word_b >> 24) & 0x3F,
        (face_data.word_e >> 16) & 0x3F,
        (face_data.word_e >> 22) & 0x3F,
    );

    let bot_left_lighting = vec3<u32>(
        (face_data.word_c >> 12) & 0x3F,
        (face_data.word_c >> 18) & 0x3F,
        (face_data.word_c >> 24) & 0x3F,
    );

    let bot_right_lighting = vec3<u32>(
        (face_data.word_d) & 0x3F,
        (face_data.word_d >> 6) & 0x3F,
        (face_data.word_d >> 12) & 0x3F,
    );

    return array<vec3<u32>, 4>(
        top_left_lighting,
        top_right_lighting,
        bot_left_lighting,
        bot_right_lighting
    );
}

fn unpack_face_ao(face_data: GPUVoxelFaceData) -> array<u32, 4> {
    let tl_ao = face_data.word_a >> 30;
    let tr_ao = face_data.word_b >> 30;
    let bl_ao = face_data.word_c >> 30;
    let br_ao = face_data.word_d >> 30;
    return array<u32, 4>(tl_ao, tr_ao, br_ao, bl_ao);
}
