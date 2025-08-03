use crate::types::Transform;

#[derive(Default)]
pub struct Camera {
    pub transform: Transform,
    pub target: glam::Vec3,
    pub properties: CameraProperties,
}

impl Camera {
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.properties.aspect_ratio = aspect_ratio;
    }
}

pub struct CameraProperties {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect_ratio: f32,
}

impl Default for CameraProperties {
    fn default() -> Self {
        Self {
            fov: 70f32.to_radians(),
            near: 0.1,
            far: 1000.0,
            aspect_ratio: 1.0,
        }
    }
}
