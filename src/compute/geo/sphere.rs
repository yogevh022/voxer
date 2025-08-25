use std::f32::consts::PI;
use crate::compute::geo::circle::Circle;
use glam::IVec3;

pub struct Sphere {}

impl Sphere {
    pub fn max_discrete_points(radius: isize) -> usize {
        let sphere_volume = 4.0 * PI / 3.0 * radius.pow(3) as f32;
        let surface_correction = 3.0 * PI * radius.pow(2) as f32;
        let constant = 2.0 * radius as f32;
        (sphere_volume + surface_correction + constant).ceil() as usize
    }

    pub fn circles_on_z<F>(position: IVec3, radius: isize, mut circle_fn: F)
    where
        F: FnMut(IVec3, isize),
    {
        let r2 = radius * radius;
        for i in (-radius)..radius {
            let dst = i - position.z as isize;
            let r2_min_dst = r2 - dst * dst;
            if r2_min_dst < 0 {
                continue;
            }
            circle_fn(IVec3::new(position.x, position.y, position.z + i as i32), r2_min_dst.isqrt());
        }
    }

    pub fn discrete_points<F>(position: IVec3, radius: isize, mut point_fn: F)
    where
        F: FnMut(IVec3),
    {
        Sphere::circles_on_z(position, radius, |circle_position, circle_radius| {
            Circle::discrete_points(circle_position.truncate(), circle_radius, |x, y| {
                point_fn(IVec3::new(x as i32, y as i32, circle_position.z));
            });
        });
    }
}
