use crate::compute::geo::circle::{Circle, CirclePointsRange};
use glam::{IVec2, IVec3};
use std::f32::consts::PI;

pub struct Sphere;

impl Sphere {
    pub fn max_discrete_points(radius: u32) -> usize {
        let sphere_volume = 4.0 * PI / 3.0 * radius.pow(3) as f32;
        let surface_correction = 3.0 * PI * radius.pow(2) as f32;
        let constant = 2.0 * radius as f32;
        (sphere_volume + surface_correction + constant).ceil() as usize
    }

    pub fn discrete_circles(radius: u32) -> impl Iterator<Item = (i32, u32)> {
        SphereCirclesRange::new(radius)
    }

    pub fn discrete_points(origin: IVec3, radius: u32) -> SpherePointsRange {
        SpherePointsRange::new(origin, radius)
    }
}

pub struct  SpherePointsRange {
    circles: SphereCirclesRange,
    points: CirclePointsRange,
    origin: IVec3,
    origin_2d: IVec2,
    delta_z: i32,
}

impl SpherePointsRange {
    pub fn new(origin: IVec3, radius: u32) -> Self {
        let mut circles = SphereCirclesRange::new(radius);
        let (delta_z, z_rad) = circles.next().unwrap(); // this unwrap never fails for all u32 values
        let origin_2d = origin.truncate();
        let points = Circle::discrete_points(origin_2d, z_rad);
        Self {
            circles,
            points,
            origin,
            origin_2d,
            delta_z,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty() && self.circles.is_empty()
    }
}

impl Iterator for SpherePointsRange {
    type Item = IVec3;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            return match self.points.next() {
                Some((x, y)) => Some(IVec3::new(x, y, self.origin.z + self.delta_z)),
                None => {
                    if let Some((d, r)) = self.circles.next() {
                        self.points = Circle::discrete_points(self.origin_2d, r);
                        self.delta_z = d;
                        continue;
                    }
                    None
                }
            };
        }
    }
}

pub struct SphereCirclesRange {
    radius: i32,
    r2: i32,
    i: i32,
}

impl SphereCirclesRange {
    pub fn new(radius: u32) -> Self {
        let radius = radius as i32;
        let r2 = radius * radius;
        Self {
            radius,
            r2,
            i: -radius,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.i > self.radius
    }
}

impl Iterator for SphereCirclesRange {
    type Item = (i32, u32);
    fn next(&mut self) -> Option<Self::Item> {
        while self.i <= self.radius {
            let i = self.i;
            let i_rad_sq = self.r2 - (i * i);
            self.i += 1;
            if i_rad_sq >= 0 {
                return Some((i, (i_rad_sq as u32).isqrt()));
            }
        }
        None
    }
}