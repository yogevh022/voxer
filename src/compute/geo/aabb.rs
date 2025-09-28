use crate::compute::geo;
use glam::{IVec3, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn inf() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }

    pub fn within_aabb(a: AABB, b: AABB) -> bool {
        a.min.x <= b.max.x
            && a.max.x >= b.min.x
            && a.min.y <= b.max.y
            && a.max.y >= b.min.y
            && a.min.z <= b.max.z
            && a.max.z >= b.min.z
    }

    pub fn discrete_points<F>(&self, mut func: F)
    where
        F: FnMut(IVec3),
    {
        for x in (self.min.x as i32)..(self.max.x as i32) {
            for y in (self.min.y as i32)..(self.max.y as i32) {
                for z in (self.min.z as i32)..(self.max.z as i32) {
                    func(IVec3::new(x, y, z));
                }
            }
        }
    }
}
