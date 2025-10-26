use bytemuck::{Pod, Zeroable};
use glam::{Mat4, UVec2, Vec4};
use voxer_macros::ShaderType;
use winit::dpi::PhysicalSize;
use crate::compute::geo::{Frustum, Plane};
use crate::vtypes::Camera;

#[repr(C, align(16))]
#[derive(ShaderType, Copy, Clone, Debug, Pod, Zeroable)]
pub struct VxGPUCamera {
    view_origin: Vec4,
    view_vp: Mat4,
    culling_origin: Vec4,
    culling_view: Mat4,
    culling_proj: Mat4,
    culling_vp: Mat4,
    culling_vf: [Plane; 6],
    window_size: UVec2,
    culling_dist: u32,
    _padding: u32,
}

impl VxGPUCamera {
    pub fn new(main_camera: &Camera, culling_dist: u32, window_size: PhysicalSize<u32>) -> Self {
        let window_size = UVec2::new(window_size.width, window_size.height);

        let view_origin = main_camera.transform.position.extend(1.0); // for alignment
        let view_vp = main_camera.view_projection_matrix();

        let culling_origin = view_origin;
        let culling_view = main_camera.view_matrix();
        let culling_proj = main_camera.projection_matrix();
        let culling_vp = view_vp;
        let culling_vf = Frustum::planes(culling_vp);
        Self {
            view_origin,
            view_vp,
            culling_origin,
            culling_dist,
            culling_view,
            culling_proj,
            culling_vp,
            culling_vf,
            window_size,
            _padding: 0,
        }
    }
}