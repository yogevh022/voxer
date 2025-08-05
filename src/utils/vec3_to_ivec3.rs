use glam::{IVec3, Vec3};

pub fn vec3_to_ivec3(vec: &Vec3) -> IVec3 {
    IVec3::new(vec.x as i32, vec.y as i32, vec.z as i32)
}
