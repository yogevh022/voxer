use crate::world::types::CHUNK_DIM;
use glam::{IVec3, Vec3};
use std::f32::consts::PI;
use crate::compute::geo::Sphere;

pub fn world_to_chunk_pos(vec: Vec3) -> IVec3 {
    let chunk_pos_float = vec / CHUNK_DIM as f32;
    IVec3::new(
        chunk_pos_float.x as i32,
        chunk_pos_float.y as i32,
        chunk_pos_float.z as i32,
    )
}

pub fn chunk_to_world_pos(chunk_pos: IVec3) -> Vec3 {
    let world_pos_round = chunk_pos * CHUNK_DIM as i32;
    Vec3::new(
        world_pos_round.x as f32,
        world_pos_round.y as f32,
        world_pos_round.z as f32,
    )
}
