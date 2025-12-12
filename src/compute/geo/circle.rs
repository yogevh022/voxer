use glam::IVec2;

pub struct Circle;

impl Circle {
    fn line_intersect_hor_2p(y: i32, c: IVec2, r2: i32) -> (i32, i32) {
        // horizontal line intersection, panics if not 2 points
        let dy = y - c.y;
        let sq = r2 - (dy * dy);
        let dx = (sq as u32).isqrt() as i32; // ub if sq < 0
        let x1 = c.x - dx;
        let x2 = c.x + dx;
        (x1, x2)
    }

    fn line_intersect_ver_2p(x: i32, c: IVec2, r2: i32) -> (i32, i32) {
        // vertical line intersection, panics if not 2 points
        let dx = x - c.x;
        let sq = r2 - (dx * dx);
        let dy = (sq as u32).isqrt() as i32; // ub if sq < 0
        let y1 = c.y - dy;
        let y2 = c.y + dy;
        (y1, y2)
    }

    pub fn discrete_points(c: IVec2, radius: u32) -> CirclePointsRange {
        CirclePointsRange::new(c, radius)
    }
}

pub struct CirclePointsRange {
    center: IVec2,
    r2: i32,
    x: i32,
    y: i32,
    max_x: i32,
    max_y: i32,
}

impl CirclePointsRange {
    fn new(center: IVec2, radius: u32) -> Self {
        let radius = radius as i32;
        let r2 = radius * radius;
        let x = center.x - radius;
        let max_x = center.x + radius;
        let (y, max_y) = Circle::line_intersect_ver_2p(x, center, r2);
        Self {
            center,
            r2,
            x,
            y,
            max_x,
            max_y,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.x > self.max_x
    }
}

impl Iterator for CirclePointsRange {
    type Item = (i32, i32);
    fn next(&mut self) -> Option<Self::Item> {
        if self.x <= self.max_x {
            let out = Some((self.x, self.y));
            self.y += 1;
            if self.y > self.max_y {
                self.x += 1;
                (self.y, self.max_y) = Circle::line_intersect_ver_2p(self.x, self.center, self.r2);
            }
            out
        } else {
            None
        }
    }
}
