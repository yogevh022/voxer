use super::Plane;
use glam::{Mat4, Vec3};
use crate::compute::geo::aabb::AABB;

pub struct Frustum;

impl Frustum {
    pub fn planes(vp: Mat4) -> [Plane; 6] {
        let m = vp.to_cols_array();
        let planes = [
            Plane {
                normal: Vec3::new(m[3] + m[0], m[7] + m[4], m[11] + m[8]),
                distance: m[15] + m[12],
            }, // left
            Plane {
                normal: Vec3::new(m[3] - m[0], m[7] - m[4], m[11] - m[8]),
                distance: m[15] - m[12],
            }, // right
            Plane {
                normal: Vec3::new(m[3] + m[1], m[7] + m[5], m[11] + m[9]),
                distance: m[15] + m[13],
            }, // bottom
            Plane {
                normal: Vec3::new(m[3] - m[1], m[7] - m[5], m[11] - m[9]),
                distance: m[15] - m[13],
            }, // top
            Plane {
                normal: Vec3::new(m[3] + m[2], m[7] + m[6], m[11] + m[10]),
                distance: m[15] + m[14],
            }, // near
            Plane {
                normal: Vec3::new(m[3] - m[2], m[7] - m[6], m[11] - m[10]),
                distance: m[15] - m[14],
            }, // far
        ];
        planes.map(|mut p| {
            let len = p.normal.length();
            p.normal /= len;
            p.distance /= len;
            p
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
            .all(|plane| plane.normal.dot(point) + plane.distance >= -1e-3)
    }

    pub fn sphere_within_frustum(center: Vec3, radius: f32, planes: &[Plane; 6]) -> bool {
        planes.iter().all(|plane| plane.normal.dot(center) + plane.distance < -radius)
    }

    pub fn aabb_within_frustum(min: Vec3, max: Vec3, planes: &[Plane; 6]) -> bool {
        for plane in planes {
            // compare with closest point to frustum
            let p = Vec3::new(
                if plane.normal.x >= 0.0 { max.x } else { min.x },
                if plane.normal.y >= 0.0 { max.y } else { min.y },
                if plane.normal.z >= 0.0 { max.z } else { min.z },
            );
            if plane.normal.dot(p) + plane.distance < 0.0 {
                return false;
            }
        }
        true
    }
}
