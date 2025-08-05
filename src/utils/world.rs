use crate::utils;
use crate::worldgen::types::CHUNK_SIZE;
use glam::Vec3;

pub(crate) fn world_to_chunk_pos(vec: Vec3) -> Vec3 {
    vec / CHUNK_SIZE as f32
}

pub(crate) fn chunk_to_world_pos(vec: Vec3) -> Vec3 {
    vec * CHUNK_SIZE as f32
}

pub(crate) fn get_relevant_hashable_chunk_pos(vec: Vec3) -> Vec<(i32, i32, i32)> {
    let mut chunk_positions = Vec::new();
    const render_distance: u32 = 10; // fixme temp const location
    let center_around_p = Vec3::new(
        vec.x.floor() - (render_distance / 2) as f32,
        vec.y.floor() - (render_distance / 2) as f32,
        vec.z.floor() - (render_distance / 2) as f32,
    );
    for x in 0..render_distance {
        for y in 0..render_distance {
            for z in 0..render_distance {
                let c_pos = center_around_p + Vec3::new(x as f32, y as f32, z as f32);
                chunk_positions.push(utils::vec3_to_i32_tuple(&c_pos));
            }
        }
    }
    chunk_positions
}
