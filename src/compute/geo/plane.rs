use glam::Vec3;

#[derive(Default, Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    pub fn intersection(p1: Plane, p2: Plane, p3: Plane) -> Option<Vec3> {
        let det = p1.normal.dot(p2.normal.cross(p3.normal));
        if det.abs() < 1e-6 { return None; }

        Some((p2.normal.cross(p3.normal) * (-p1.distance) +
            p3.normal.cross(p1.normal) * (-p2.distance) +
            p1.normal.cross(p2.normal) * (-p3.distance)) / det)
    }
}