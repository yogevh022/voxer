use crate::worldgen::types::World;
use glam::{IVec3, Mat4};

pub fn model_matrix(c_pos: &IVec3) -> Mat4 {
    let chunk_world_position = World::chunk_to_world_pos(c_pos);
    Mat4::from_translation(chunk_world_position)
}
