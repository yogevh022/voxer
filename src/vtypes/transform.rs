use glam;
use std::default::Default;

pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl Transform {
    pub fn up(&self) -> glam::Vec3 {
        self.rotation * glam::Vec3::Y
    }

    pub fn forward(&self) -> glam::Vec3 {
        self.rotation * glam::Vec3::Z
    }

    pub fn right(&self) -> glam::Vec3 {
        self.rotation * -glam::Vec3::X
    }

    pub fn from_vec3(vec: glam::Vec3) -> Self {
        Self {
            position: vec,
            ..Default::default()
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }
}
