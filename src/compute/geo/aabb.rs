use glam::{IVec3, Vec3};
use range3d::Range3D;

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

    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    pub fn within_aabb(a: AABB, b: AABB) -> bool {
        a.min.x <= b.max.x
            && a.max.x >= b.min.x
            && a.min.y <= b.max.y
            && a.max.y >= b.min.y
            && a.min.z <= b.max.z
            && a.max.z >= b.min.z
    }

    pub fn diff_out(&self, other: AABB, out: &mut Vec<AABB>) {
        let mut diffs: [Option<AABB>; 6] = [None; 6];

        if self.min.x < other.min.x {
            let mut diff = *self;
            diff.max.x = other.min.x;
            diffs[0] = Some(diff);
        }

        if self.max.x > other.max.x {
            let mut diff = *self;
            diff.min.x = other.max.x;
            diffs[1] = Some(diff);
        }

        if self.min.y < other.min.y {
            let mut diff = *self;
            diff.max.y = other.min.y;
            if let Some(dx) = diffs[0] {
                diff.min.x = dx.max.x;
            }
            if let Some(dx) = diffs[1] {
                diff.max.x = dx.min.x;
            }
            diffs[2] = Some(diff);
        }

        if self.max.y > other.max.y {
            let mut diff = *self;
            diff.min.y = other.max.y;
            if let Some(dx) = diffs[0] {
                diff.min.x = dx.max.x;
            }
            if let Some(dx) = diffs[1] {
                diff.max.x = dx.min.x;
            }
            diffs[3] = Some(diff);
        }

        if self.min.z < other.min.z {
            let mut diff = *self;
            diff.max.z = other.min.z;
            if let Some(dx) = diffs[0] {
                diff.min.x = dx.max.x;
            }
            if let Some(dx) = diffs[1] {
                diff.max.x = dx.min.x;
            }
            if let Some(my) = diffs[2] {
                diff.min.y = my.max.y;
            }
            if let Some(my) = diffs[3] {
                diff.max.y = my.min.y;
            }
            diffs[4] = Some(diff);
        }

        if self.max.z > other.max.z {
            let mut diff = *self;
            diff.min.z = other.max.z;
            if let Some(dx) = diffs[0] {
                diff.min.x = dx.max.x;
            }
            if let Some(dx) = diffs[1] {
                diff.max.x = dx.min.x;
            }
            if let Some(my) = diffs[2] {
                diff.min.y = my.max.y;
            }
            if let Some(my) = diffs[3] {
                diff.max.y = my.min.y;
            }
            diffs[5] = Some(diff);
        }

        diffs.into_iter().flatten().for_each(|d| out.push(d));
    }

    pub fn diff(&self, other: AABB) -> Vec<AABB> {
        let mut out: Vec<AABB> = Vec::with_capacity(6);
        self.diff_out(other, &mut out);
        out
    }

    pub fn sym_diff_out(
        &self,
        right: AABB,
        left_only_out: &mut Vec<AABB>,
        right_only_out: &mut Vec<AABB>,
    ) {
        let mut in_left: [Option<AABB>; 6] = [None; 6];
        let mut in_right: [Option<AABB>; 6] = [None; 6];

        match (self.min.x, right.min.x) {
            (lmx, rmx) if lmx < rmx => {
                let mut diff = *self;
                diff.max.x = rmx;
                in_left[0] = Some(diff);
            }
            (lmx, rmx) if lmx > rmx => {
                let mut diff = right;
                diff.max.x = lmx;
                in_right[0] = Some(diff);
            }
            _ => (),
        };

        match (self.max.x, right.max.x) {
            (lmx, rmx) if lmx < rmx => {
                let mut diff = right;
                diff.min.x = lmx;
                in_right[1] = Some(diff);
            }
            (lmx, rmx) if lmx > rmx => {
                let mut diff = *self;
                diff.min.x = rmx;
                in_left[1] = Some(diff);
            }
            _ => (),
        };

        match (self.min.y, right.min.y) {
            (lmy, rmy) if lmy < rmy => {
                let mut diff = *self;
                diff.max.y = rmy;
                if let Some(mx) = in_left[0] {
                    diff.min.x = mx.max.x;
                }
                if let Some(mx) = in_left[1] {
                    diff.max.x = mx.min.x;
                }
                in_left[2] = Some(diff);
            }
            (lmy, rmy) if lmy > rmy => {
                let mut diff = right;
                diff.max.y = lmy;
                if let Some(mx) = in_right[0] {
                    diff.min.x = mx.max.x;
                }
                if let Some(mx) = in_right[1] {
                    diff.max.x = mx.min.x;
                }
                in_right[2] = Some(diff);
            }
            _ => (),
        };

        match (self.max.y, right.max.y) {
            (lmy, rmy) if lmy < rmy => {
                let mut diff = right;
                diff.min.y = lmy;
                if let Some(mx) = in_right[0] {
                    diff.min.x = mx.max.x;
                }
                if let Some(mx) = in_right[1] {
                    diff.max.x = mx.min.x;
                }
                in_right[3] = Some(diff);
            }
            (lmy, rmy) if lmy > rmy => {
                let mut diff = *self;
                diff.min.y = rmy;
                if let Some(mx) = in_left[0] {
                    diff.min.x = mx.max.x;
                }
                if let Some(mx) = in_left[1] {
                    diff.max.x = mx.min.x;
                }
                in_left[3] = Some(diff);
            }
            _ => (),
        };

        match (self.min.z, right.min.z) {
            (lmz, rmz) if lmz < rmz => {
                let mut diff = *self;
                diff.max.z = rmz;
                if let Some(mx) = in_left[0] {
                    diff.min.x = mx.max.x;
                }
                if let Some(mx) = in_left[1] {
                    diff.max.x = mx.min.x;
                }
                if let Some(my) = in_left[2] {
                    diff.min.y = my.max.y;
                }
                if let Some(my) = in_left[3] {
                    diff.max.y = my.min.y;
                }
                in_left[4] = Some(diff);
            }
            (lmz, rmz) if lmz > rmz => {
                let mut diff = right;
                diff.max.z = lmz;
                if let Some(mx) = in_right[0] {
                    diff.min.x = mx.max.x;
                }
                if let Some(mx) = in_right[1] {
                    diff.max.x = mx.min.x;
                }
                if let Some(my) = in_right[2] {
                    diff.min.y = my.max.y;
                }
                if let Some(my) = in_right[3] {
                    diff.max.y = my.min.y;
                }
                in_right[4] = Some(diff);
            }
            _ => (),
        };

        match (self.max.z, right.max.z) {
            (lmz, rmz) if lmz < rmz => {
                let mut diff = right;
                diff.min.z = lmz;
                if let Some(mx) = in_right[0] {
                    diff.min.x = mx.max.x;
                }
                if let Some(mx) = in_right[1] {
                    diff.max.x = mx.min.x;
                }
                if let Some(my) = in_right[2] {
                    diff.min.y = my.max.y;
                }
                if let Some(my) = in_right[3] {
                    diff.max.y = my.min.y;
                }
                in_right[5] = Some(diff);
            }
            (lmz, rmz) if lmz > rmz => {
                let mut diff = *self;
                diff.min.z = rmz;
                if let Some(mx) = in_left[0] {
                    diff.min.x = mx.max.x;
                }
                if let Some(mx) = in_left[1] {
                    diff.max.x = mx.min.x;
                }
                if let Some(my) = in_left[2] {
                    diff.min.y = my.max.y;
                }
                if let Some(my) = in_left[3] {
                    diff.max.y = my.min.y;
                }
                in_left[5] = Some(diff);
            }
            _ => (),
        };

        for i in 0..6 {
            if let Some(a) = in_left[i] {
                left_only_out.push(a);
            }
            if let Some(a) = in_right[i] {
                right_only_out.push(a);
            }
        }
    }

    pub fn sym_diff(&self, right: AABB) -> (Vec<AABB>, Vec<AABB>) {
        let mut out_left: Vec<AABB> = Vec::with_capacity(6);
        let mut out_right: Vec<AABB> = Vec::with_capacity(6);
        self.sym_diff_out(right, &mut out_left, &mut out_right);
        (out_left, out_right)
    }

    pub fn discrete_points(&self) -> Range3D {
        Range3D::new(
            self.min.x as isize,
            self.min.y as isize,
            self.min.z as isize,
            self.max.x as isize,
            self.max.y as isize,
            self.max.z as isize,
        )
    }
}
