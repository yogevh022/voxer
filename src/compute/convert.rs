use glam::{IVec3, Vec3};

pub fn vec3_ivec3(v: Vec3) -> IVec3 {
    IVec3::new(v.x as i32, v.y as i32, v.z as i32)
}