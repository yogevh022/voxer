use glam::{IVec2};

pub struct Circle;

impl Circle {
    pub fn line_intersect_hor_2p(y: isize, c: IVec2, r2: isize) -> (isize, isize) {
        // horizontal line intersection, panics if not 2 points
        let dy = y - c.y as isize;
        let sq = r2 - (dy * dy);
        let dx = sq.isqrt();
        let x1 = c.x as isize - dx;
        let x2 = c.x as isize + dx;
        (x1, x2)
    }

    pub fn discrete_points<F>(c: IVec2, radius: isize, mut point_fn: F)
    where
        F: FnMut(isize, isize),
    {
        let r2 = radius * radius;
        for y in (c.y as isize - radius)..=(c.y as isize + radius) {
            let (ix1, ix2) = Circle::line_intersect_hor_2p(y, c, r2);
            (ix1..=ix2).for_each(|x| point_fn(x, y,));
        }
    }
}
