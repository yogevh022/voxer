use super::Plane;
use crate::compute::geo::aabb::AABB;
use glam::{Mat4, Vec3};

pub struct Frustum;

impl Frustum {
    pub fn planes(vp: Mat4) -> [Plane; 6] {
        let m = vp.to_cols_array();
        let planes = [
            Plane::from_xyzw(m[3] + m[0], m[7] + m[4], m[11] + m[8], m[15] + m[12]), // left
            Plane::from_xyzw(m[3] - m[0], m[7] - m[4], m[11] - m[8], m[15] - m[12]), // right
            Plane::from_xyzw(m[3] + m[1], m[7] + m[5], m[11] + m[9], m[15] + m[13]), // bottom
            Plane::from_xyzw(m[3] - m[1], m[7] - m[5], m[11] - m[9], m[15] - m[13]), // top
            Plane::from_xyzw(m[3] + m[2], m[7] + m[6], m[11] + m[10], m[15] + m[14]), // near
            Plane::from_xyzw(m[3] - m[2], m[7] - m[6], m[11] - m[10], m[15] - m[14]), // far
        ];
        planes.map(|p| {
            let len = p.truncate().length();
            Plane::from_xyzw(p.x / len, p.y / len, p.z / len, p.w / len)
        })
    }

    pub fn aabb(planes: &[Plane; 6]) -> AABB {
        let corners = [
            // near plane
            Plane::intersection(planes[0], planes[2], planes[4]),
            Plane::intersection(planes[0], planes[2], planes[5]),
            Plane::intersection(planes[0], planes[3], planes[4]),
            Plane::intersection(planes[0], planes[3], planes[5]),
            // far plane
            Plane::intersection(planes[1], planes[2], planes[4]),
            Plane::intersection(planes[1], planes[2], planes[5]),
            Plane::intersection(planes[1], planes[3], planes[4]),
            Plane::intersection(planes[1], planes[3], planes[5]),
        ];

        let mut aabb = AABB::inf();
        for corner in corners {
            if let Some(point) = corner {
                if Frustum::point_within_frustum(point, planes) {
                    aabb.min = aabb.min.min(point);
                    aabb.max = aabb.max.max(point);
                }
            }
        }
        aabb
    }

    pub fn point_within_frustum(point: Vec3, planes: &[Plane; 6]) -> bool {
        planes
            .iter()
            .all(|plane| plane.truncate().dot(point) + plane.w >= -1e-3)
    }

    pub fn sphere_within_frustum(center: Vec3, radius: f32, planes: &[Plane; 6]) -> bool {
        planes
            .iter()
            .all(|plane| plane.truncate().dot(center) + plane.w < -radius)
    }

    pub fn aabb_within_frustum(min: Vec3, max: Vec3, planes: &[Plane; 6]) -> bool {
        for plane in planes {
            // compare with closest point to frustum
            let p = Vec3::new(
                if plane.x >= 0.0 { max.x } else { min.x },
                if plane.y >= 0.0 { max.y } else { min.y },
                if plane.z >= 0.0 { max.z } else { min.z },
            );
            if plane.truncate().dot(p) + plane.w < 0.0 {
                return false;
            }
        }
        true
    }
}
