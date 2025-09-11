use crate::vtypes::scene_object::VoxerObject;
use crate::vtypes::{Transform, Voxer};
use glam::{Mat4, Quat};

#[derive(Default)]
pub struct Camera {
    pub transform: Transform,
    pub frustum: ViewFrustum,
}

impl Camera {
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.frustum.aspect_ratio = aspect_ratio;
    }

    pub fn view_projection(&self) -> Mat4 {
        Mat4::perspective_rh(
            self.frustum.fov,
            self.frustum.aspect_ratio,
            self.frustum.near,
            self.frustum.far,
        ) * Mat4::look_to_rh(
            self.transform.position,
            self.transform.forward(),
            self.transform.up(),
        )
    }

    pub fn chunk_view_projection(&self, render_distance: f32) -> Mat4 {
        Mat4::perspective_rh(
            self.frustum.fov,
            self.frustum.aspect_ratio,
            self.frustum.near,
            render_distance,
        ) * Mat4::look_to_rh(
            self.transform.position,
            self.transform.forward(),
            self.transform.up(),
        )
    }
}

pub struct ViewFrustum {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect_ratio: f32,
}

impl ViewFrustum {
    pub fn half_height_at_depth(&self, depth: f32) -> f32 {
        depth * 0.5f32.tan()
    }
    
    pub fn half_width_at_depth(&self, depth: f32) -> f32 {
        self.aspect_ratio * self.half_height_at_depth(depth)
    }
}

impl Default for ViewFrustum {
    fn default() -> Self {
        Self {
            fov: 70f32.to_radians(),
            near: 0.1,
            far: 1000.0,
            aspect_ratio: 1.0,
        }
    }
}

pub struct CameraController {
    pitch_angle: f32,
    sensitivity: f64,
    pub yaw: Quat,
    pub pitch: Quat,
}

impl CameraController {
    pub fn with_sensitivity(sensitivity: f64) -> Self {
        Self {
            sensitivity,
            ..Default::default()
        }
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            sensitivity: 0.005,
            pitch_angle: 0.0,
            yaw: Quat::IDENTITY,
            pitch: Quat::IDENTITY,
        }
    }
}

impl CameraController {
    pub fn look(&mut self, delta: (f64, f64)) {
        self.pitch_angle =
            (self.pitch_angle + delta.1 as f32).clamp(-89f32.to_radians(), 89f32.to_radians());
        self.yaw *= Quat::from_axis_angle(glam::Vec3::Y, -delta.0 as f32);
        self.pitch = Quat::from_axis_angle(glam::Vec3::X, self.pitch_angle);
    }
    pub fn get_rotation(&self) -> Quat {
        self.yaw * self.pitch
    }
}

impl VoxerObject for CameraController {
    fn update(&mut self, voxer: &mut Voxer) {
        let input = voxer.input.read();
        self.look((
            input.mouse.delta[0] * self.sensitivity,
            input.mouse.delta[1] * self.sensitivity,
        ));
        voxer.camera.transform.rotation = self.get_rotation();
    }
}
