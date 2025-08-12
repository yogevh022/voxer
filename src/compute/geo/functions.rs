use crate::world::types::CHUNK_DIM;
use glam::{IVec3, Vec3};
use std::collections::HashSet;
use std::f32::consts::PI;

#[inline]
pub fn xyz_distance_squared(x: f32, y: f32, z: f32, cx: f32, cy: f32, cz: f32) -> f32 {
    let dx = x - cx;
    let dy = y - cy;
    let dz = z - cz;
    dx * dx + dy * dy + dz * dz
}

pub fn discrete_sphere_pts(pos: &Vec3, radius: f32) -> Vec<IVec3> {
    let points_upper_bound = max_discrete_sphere_pts(radius); // make sure no vec reallocations are needed
    let mut points = Vec::with_capacity(points_upper_bound);

    let radius_squared = radius * radius;
    let min_x = (pos.x - radius) as i32;
    let max_x = (pos.x + radius) as i32;
    let min_y = (pos.y - radius) as i32;
    let max_y = (pos.y + radius) as i32;
    let min_z = (pos.z - radius) as i32;
    let max_z = (pos.z + radius) as i32;

    for x in min_x..max_x {
        for y in min_y..max_y {
            for z in min_z..max_z {
                if xyz_distance_squared(x as f32, y as f32, z as f32, pos.x, pos.y, pos.z)
                    <= radius_squared
                {
                    points.push(IVec3::new(x, y, z));
                }
            }
        }
    }
    points
}

#[inline(always)]
pub fn max_discrete_sphere_pts(radius: f32) -> usize {
    let sphere_volume = 4.0 * PI / 3.0 * radius.powi(3);
    let surface_correction = 3.0 * PI * radius.powi(2);
    let constant = 2.0 * radius;
    (sphere_volume + surface_correction + constant).ceil() as usize
}

pub fn world_to_chunk_pos(vec: &Vec3) -> IVec3 {
    let chunk_pos_float = vec / CHUNK_DIM as f32;
    IVec3::new(
        chunk_pos_float.x as i32,
        chunk_pos_float.y as i32,
        chunk_pos_float.z as i32,
    )
}

pub fn chunk_to_world_pos(chunk_pos: &IVec3) -> Vec3 {
    let world_pos_round = chunk_pos * CHUNK_DIM as i32;
    Vec3::new(
        world_pos_round.x as f32,
        world_pos_round.y as f32,
        world_pos_round.z as f32,
    )
}
