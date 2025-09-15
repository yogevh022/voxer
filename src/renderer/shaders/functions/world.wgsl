fn chunk_to_world_position(chunk_position: vec3<i32>) -> vec3<f32> {
    return vec3<f32>(chunk_position * i32(VCHUNK_DIM));
}