use glam::{IVec3, Vec3};
use std::collections::HashSet;
use std::f32::consts::PI;

#[inline]
pub fn is_within_sphere(pos: Vec3, sphere_pos: Vec3, sphere_radius: f32) -> bool {
    (pos - sphere_pos).length_squared() <= sphere_radius * sphere_radius
}

pub fn discrete_sphere_pts(pos: Vec3, radius: f32) -> HashSet<IVec3> {
    let points_upper_bound = max_discrete_sphere_pts(radius); // make sure no vec reallocations are needed
    let mut points = HashSet::with_capacity(points_upper_bound);
    for x in (pos.x - radius) as i32..(pos.x + radius) as i32 {
        for y in (pos.y - radius) as i32..(pos.y + radius) as i32 {
            for z in (pos.z - radius) as i32..(pos.z + radius) as i32 {
                if is_within_sphere(Vec3::new(x as f32, y as f32, z as f32), pos, radius) {
                    points.insert(IVec3::new(x, y, z));
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
