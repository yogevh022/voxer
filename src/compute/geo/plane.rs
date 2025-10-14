use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};
use std::ops::{Deref, DerefMut};
use voxer_macros::ShaderType;

#[repr(C, align(16))]
#[derive(ShaderType, Default, Debug, Clone, Copy, Pod, Zeroable)]
pub struct Plane {
    equation: Vec4,
}

impl Deref for Plane {
    type Target = Vec4;

    fn deref(&self) -> &Self::Target {
        &self.equation
    }
}

impl DerefMut for Plane {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.equation
    }
}

impl Plane {
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self {
            equation: normal.extend(distance),
        }
    }
    
    pub fn from_xyzw(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            equation: Vec4::new(x, y, z, w),
        }
    }

    pub fn intersection(p1: Plane, p2: Plane, p3: Plane) -> Option<Vec3> {
        let n1 = Vec3::new(p1.equation.x, p1.equation.y, p1.equation.z);
        let n2 = Vec3::new(p2.equation.x, p2.equation.y, p2.equation.z);
        let n3 = Vec3::new(p3.equation.x, p3.equation.y, p3.equation.z);

        let det = n1.dot(n2.cross(n3));
        if det.abs() < 1e-6 {
            return None;
        }

        Some(
            (n2.cross(n3) * (-p1.equation.w)
                + n3.cross(n1) * (-p2.equation.w)
                + n1.cross(n2) * (-p3.equation.w))
                / det,
        )
    }
}
