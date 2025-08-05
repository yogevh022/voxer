use glam::Vec3;

pub fn vec3_to_i32_tuple(vec: &Vec3) -> (i32, i32, i32) {
    (vec.x as i32, vec.y as i32, vec.z as i32)
}
