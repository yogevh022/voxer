use glam::Vec3;

#[derive(Default, Debug, Clone, Copy)]
pub struct Plane {
    pub n: Vec3,
    pub d: f32,
}