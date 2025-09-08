use super::Plane;
use glam::{Mat4, Vec3};

pub struct Frustum;

impl Frustum {
    pub fn planes(vp: Mat4) -> [Plane; 6] {
        let m = vp.to_cols_array();
        let planes = [
            Plane {
                n: Vec3::new(m[3] + m[0], m[7] + m[4], m[11] + m[8]),
                d: m[15] + m[12],
            }, // left
            Plane {
                n: Vec3::new(m[3] - m[0], m[7] - m[4], m[11] - m[8]),
                d: m[15] - m[12],
            }, // right
            Plane {
                n: Vec3::new(m[3] + m[1], m[7] + m[5], m[11] + m[9]),
                d: m[15] + m[13],
            }, // bottom
            Plane {
                n: Vec3::new(m[3] - m[1], m[7] - m[5], m[11] - m[9]),
                d: m[15] - m[13],
            }, // top
            Plane {
                n: Vec3::new(m[3] + m[2], m[7] + m[6], m[11] + m[10]),
                d: m[15] + m[14],
            }, // near
            Plane {
                n: Vec3::new(m[3] - m[2], m[7] - m[6], m[11] - m[10]),
                d: m[15] - m[14],
            }, // far
        ];
        planes.map(|mut p| {
            let len = p.n.length();
            p.n /= len;
            p.d /= len;
            p
        })
    }

    pub fn is_aabb_within_frustum(min: Vec3, max: Vec3, planes: &[Plane; 6]) -> bool {
        for plane in planes {
            // compare with closest point to frustum
            let p = Vec3::new(
                if plane.n.x >= 0.0 { max.x } else { min.x },
                if plane.n.y >= 0.0 { max.y } else { min.y },
                if plane.n.z >= 0.0 { max.z } else { min.z },
            );
            if plane.n.dot(p) + plane.d < 0.0 {
                return false;
            }
        }
        true
    }

    // pub fn frustum_corners_world(vp: Mat4) -> [Vec3; 4] {
    //     let inv_vp = vp.inverse();
    //     let ndc = [
    //         Vec4::new(-1.0,  1.0, 0.0, 1.0), // near tl
    //         Vec4::new( 1.0, -1.0, 0.0, 1.0), // near br
    //         Vec4::new(-1.0,  1.0, 1.0, 1.0), // far tl
    //         Vec4::new( 1.0, -1.0, 1.0, 1.0), // far br
    //     ];
    //
    //     let mut corners = [Vec3::ZERO; 4];
    //     for (i, &c) in ndc.iter().enumerate() {
    //         let world = inv_vp * c;
    //         corners[i] = world.xyz() / world.w;
    //     }
    //     corners
    // }

    // pub fn map_points_to_depth<F>(depth: usize, vp: Mat4)
    // {
    //     let mut points = Vec::new(); // todo preallocate with capacity
    //     let world_corners = Frustum::frustum_corners_world(vp);
    //     for z in 0..=depth {
    //         let depth_lerp = z as f32 / depth as f32;
    //         let corners = [
    //             world_corners[0].lerp(world_corners[2], depth_lerp), // tl
    //             world_corners[1].lerp(world_corners[3], depth_lerp), // br
    //         ];
    //         Rect::map_points_within_rect(corners[0], corners[1], |point| {
    //             points.push(point.extend(z as i32));
    //         });
    //     }
    // }
}
