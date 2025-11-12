use glam::{IVec3, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn inf() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }

    pub fn zero() -> Self {
        Self {
            min: Vec3::splat(0.0),
            max: Vec3::splat(0.0),
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

    pub fn sym_diff_out(left: AABB, right: AABB, left_only_out: &mut Vec<AABB>, right_only_out: &mut Vec<AABB>) {
        let mut in_left: [Option<AABB>; 6] = [None; 6];
        let mut in_right: [Option<AABB>; 6] = [None; 6];

        match (left.min.x, right.min.x) {
            (lmx, rmx) if lmx < rmx => {
                let mut q = left.clone();
                q.max.x = rmx;
                in_left[0] = Some(q);
            }
            (lmx, rmx) if lmx > rmx => {
                let mut q = right.clone();
                q.max.x = lmx;
                in_right[0] = Some(q);
            }
            _ => ()
        };

        match (left.max.x, right.max.x) {
            (lmx, rmx) if lmx < rmx => {
                let mut q = right.clone();
                q.min.x = lmx;
                in_right[1] = Some(q);
            }
            (lmx, rmx) if lmx > rmx => {
                let mut q = left.clone();
                q.min.x = rmx;
                in_left[1] = Some(q);
            }
            _ => ()
        };

        match (left.min.y, right.min.y) {
            (lmy, rmy) if lmy < rmy => {
                let mut q = left.clone();
                q.max.y = rmy;
                if let Some(mx) = in_left[0] {
                    q.min.x = mx.max.x;
                }
                if let Some(mx) = in_left[1] {
                    q.max.x = mx.min.x;
                }
                in_left[2] = Some(q);
            }
            (lmy, rmy) if lmy > rmy => {
                let mut q = right.clone();
                q.max.y = lmy;
                if let Some(mx) = in_right[0] {
                    q.min.x = mx.max.x;
                }
                if let Some(mx) = in_right[1] {
                    q.max.x = mx.min.x;
                }
                in_right[2] = Some(q);
            }
            _ => ()
        };

        match (left.max.y, right.max.y) {
            (lmy, rmy) if lmy < rmy => {
                let mut q = right.clone();
                q.min.y = lmy;
                if let Some(mx) = in_right[0] {
                    q.min.x = mx.max.x;
                }
                if let Some(mx) = in_right[1] {
                    q.max.x = mx.min.x;
                }
                in_right[3] = Some(q);
            }
            (lmy, rmy) if lmy > rmy => {
                let mut q = left.clone();
                q.min.y = rmy;
                if let Some(mx) = in_left[0] {
                    q.min.x = mx.max.x;
                }
                if let Some(mx) = in_left[1] {
                    q.max.x = mx.min.x;
                }
                in_left[3] = Some(q);
            }
            _ => ()
        };

        match (left.min.z, right.min.z) {
            (lmz, rmz) if lmz < rmz => {
                let mut q = left.clone();
                q.max.z = rmz;
                if let Some(mx) = in_left[0] {
                    q.min.x = mx.max.x;
                }
                if let Some(mx) = in_left[1] {
                    q.max.x = mx.min.x;
                }
                if let Some(my) = in_left[2] {
                    q.min.y = my.max.y;
                }
                if let Some(my) = in_left[3] {
                    q.max.y = my.min.y;
                }
                in_left[4] = Some(q);
            }
            (lmz, rmz) if lmz > rmz => {
                let mut q = right.clone();
                q.max.z = lmz;
                if let Some(mx) = in_right[0] {
                    q.min.x = mx.max.x;
                }
                if let Some(mx) = in_right[1] {
                    q.max.x = mx.min.x;
                }
                if let Some(my) = in_right[2] {
                    q.min.y = my.max.y;
                }
                if let Some(my) = in_right[3] {
                    q.max.y = my.min.y;
                }
                in_right[4] = Some(q);
            }
            _ => ()
        };

        match (left.max.z, right.max.z) {
            (lmz, rmz) if lmz < rmz => {
                let mut q = right.clone();
                q.min.z = lmz;
                if let Some(mx) = in_right[0] {
                    q.min.x = mx.max.x;
                }
                if let Some(mx) = in_right[1] {
                    q.max.x = mx.min.x;
                }
                if let Some(my) = in_right[2] {
                    q.min.y = my.max.y;
                }
                if let Some(my) = in_right[3] {
                    q.max.y = my.min.y;
                }
                in_right[5] = Some(q);
            }
            (lmz, rmz) if lmz > rmz => {
                let mut q = left.clone();
                q.min.z = rmz;
                if let Some(mx) = in_left[0] {
                    q.min.x = mx.max.x;
                }
                if let Some(mx) = in_left[1] {
                    q.max.x = mx.min.x;
                }
                if let Some(my) = in_left[2] {
                    q.min.y = my.max.y;
                }
                if let Some(my) = in_left[3] {
                    q.max.y = my.min.y;
                }
                in_left[5] = Some(q);
            }
            _ => ()
        };

        left_only_out.clear();
        right_only_out.clear();

        for i in 0..6 {
            if let Some(a) = in_left[i] {
                left_only_out.push(a);
            }
            if let Some(a) = in_right[i] {
                right_only_out.push(a);
            }
        }
    }

    pub fn sym_diff(left: AABB, right: AABB) -> (Vec<AABB>, Vec<AABB>) {
        let mut out_left: Vec<AABB> = Vec::with_capacity(6);
        let mut out_right: Vec<AABB> = Vec::with_capacity(6);
        Self::sym_diff_out(left, right, &mut out_left, &mut out_right);
        (out_left, out_right)
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
