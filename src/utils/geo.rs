use glam::{IVec3, Vec3};

#[inline]
pub fn is_within_sphere(pos: Vec3, sphere_pos: Vec3, sphere_radius: f32) -> bool {
    (pos - sphere_pos).length_squared() <= sphere_radius * sphere_radius
}

pub fn discrete_points_within_sphere(pos: Vec3, radius: f32) -> Vec<IVec3> {
    let diameter = (2f32 * radius + 1f32) as usize; // >upper bound so no realloc
    let points_capacity = diameter * diameter * diameter;
    let mut points = Vec::with_capacity(points_capacity);

    for x in (pos.x - radius) as i32..(pos.x + radius) as i32 {
        for y in (pos.y - radius) as i32..(pos.y + radius) as i32 {
            for z in (pos.z - radius) as i32..(pos.z + radius) as i32 {
                if is_within_sphere(Vec3::new(x as f32, y as f32, z as f32), pos, radius) {
                    points.push(IVec3::new(x, y, z));
                }
            }
        }
    }
    points
}
