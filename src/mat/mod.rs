use glam::{Mat4, Quat, Vec3};

pub fn model_to_world_matrix(position: Vec3, rotation: Quat, scale: Vec3) -> Mat4 {
    Mat4::from_translation(position) * Mat4::from_quat(rotation) * Mat4::from_scale(scale)
}
