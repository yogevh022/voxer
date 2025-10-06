use std::mem::MaybeUninit;
use glam::IVec3;
use std::ops::Range;

pub struct IVec3Iter {
    x_range: Range<i32>,
    y_range: Range<i32>,
    z_range: Range<i32>,
    current: IVec3,
}

impl IVec3Iter {
    pub fn new(x_range: Range<i32>, y_range: Range<i32>, z_range: Range<i32>) -> Self {
        Self {
            current: IVec3::new(x_range.start, y_range.start, z_range.start),
            x_range,
            y_range,
            z_range,
        }
    }
}

impl Iterator for IVec3Iter {
    type Item = IVec3;

    #[inline]
    fn next(&mut self) -> Option<IVec3> {
        if self.current.z >= self.z_range.end {
            return None;
        }
        let result = self.current;
        self.current.x += 1;
        if self.current.x >= self.x_range.end {
            self.current.x = self.x_range.start;
            self.current.y += 1;
            if self.current.y >= self.y_range.end {
                self.current.y = self.y_range.start;
                self.current.z += 1;
            }
        }
        Some(result)
    }
}

pub fn ivec3_with_adjacent_positions(origin: IVec3) -> [IVec3; 7] {
    const ADJ_OFFSETS: [IVec3; 6] = [
        IVec3::new(-1, 0, 0),
        IVec3::new(0, -1, 0),
        IVec3::new(0, 0, -1),
        IVec3::new(1, 0, 0),
        IVec3::new(0, 1, 0),
        IVec3::new(0, 0, 1),
    ];
    let mut result: [MaybeUninit<IVec3>; 7] = unsafe { MaybeUninit::uninit().assume_init() };
    for i in 0..6 {
        result[i] = MaybeUninit::new(origin + ADJ_OFFSETS[i]);
    }
    result[6] = MaybeUninit::new(origin);
    unsafe { std::mem::transmute(result) }
}
