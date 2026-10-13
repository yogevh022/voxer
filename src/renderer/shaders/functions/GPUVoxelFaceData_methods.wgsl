
struct VoxelFace {
    voxel: u32,
    face_id: u32,
}

fn unpack_face_voxel(face_data: GPUVoxelFaceData) -> VoxelFace {
    let voxel = face_data.word_a & 0xFFFFu;
    let face_id = (face_data.word_a >> 28) & 0x7;
    return VoxelFace(voxel, face_id);
}

fn unpack_face_position(face_data: GPUVoxelFaceData) -> vec3<u32> {
    return vec3<u32>(
        (face_data.word_a >> 16) & 0x0F,
        (face_data.word_a >> 20) & 0x0F,
        (face_data.word_a >> 24) & 0x0F,
    );
}
