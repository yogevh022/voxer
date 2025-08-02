use glam::{Mat4, Vec3, Vec4};

fn model_to_world_matrix(position: Vec3, rotation_y_deg: f32, scale: f32) -> Mat4 {
    let translation = Mat4::from_translation(position);
    let rotation = Mat4::from_rotation_y(rotation_y_deg.to_radians());
    let scaling = Mat4::from_scale(Vec3::splat(scale));
    
    translation * rotation * scaling
}

fn world_to_view_matrix(cam: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    let forward = (target - cam).normalize();
    let right = forward.cross(up).normalize();
    let up = right.cross(forward).normalize();
    
    Mat4::from_cols(
        right.extend(0.0),
        up.extend(0.0),
        forward.extend(0.0),
        Vec4::new(
            -right.dot(cam),
            -up.dot(cam),
            forward.dot(cam),
            1.0
        )
    )
}
