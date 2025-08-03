use glam::{Mat4, Vec3, Vec4};

pub fn model_to_world_matrix(position: Vec3, rotation_y_deg: f32, scale: f32) -> Mat4 {
    let translation = Mat4::from_translation(position);
    let rotation = Mat4::from_rotation_y(rotation_y_deg.to_radians());
    let scaling = Mat4::from_scale(Vec3::splat(scale));

    translation * rotation * scaling
}
