use crate::vtypes::{CameraController, Voxer};

pub trait VoxerObject {
    fn update(&mut self, voxer: &mut Voxer);
}

pub enum VObject {
    Camera(CameraController),
}

impl VoxerObject for VObject {
    fn update(&mut self, voxer: &mut Voxer) {
        match self {
            VObject::Camera(camera) => camera.update(voxer),
        }
    }
}
