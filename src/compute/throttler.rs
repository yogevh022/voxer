use glam::{IVec3, UVec3};
use std::time::{Duration, Instant};

pub trait Throttler {
    type Key;
    fn new(max_pending: usize, throttle_duration: Duration) -> Self;
    fn request(&mut self, key: Self::Key) -> bool;
    fn set_now(&mut self, now: Instant);
}

pub struct BaseThrottler {
    pub max_pending: usize,
    pub throttle_duration: Duration,
    now: Instant,
    queue: Box<[Instant]>,
}

impl Throttler for BaseThrottler {
    type Key = usize;
    fn new(max_pending: usize, throttle_duration: Duration) -> Self {
        let now = Instant::now();
        let available_now = now - throttle_duration;
        Self {
            max_pending,
            throttle_duration,
            now,
            queue: vec![available_now; max_pending].into_boxed_slice(),
        }
    }

    fn request(&mut self, key: Self::Key) -> bool {
        debug_assert!(key < self.max_pending);
        let can_request = self.now.duration_since(self.queue[key]) >= self.throttle_duration;
        if can_request {
            self.queue[key] = self.now;
        }
        can_request
    }

    fn set_now(&mut self, now: Instant) {
        self.now = now;
    }
}

pub struct SpatialThrottler {
    throttler: BaseThrottler,
    spatial_dim: IVec3,
}

impl SpatialThrottler {
    pub(crate) fn new(spatial_dim: UVec3, throttle_duration: Duration) -> Self {
        let max_pending = spatial_dim.element_product() as usize;
        Self {
            throttler: BaseThrottler::new(max_pending, throttle_duration),
            spatial_dim: spatial_dim.as_ivec3(),
        }
    }

    pub(crate) fn request(&mut self, key: IVec3) -> bool {
        let (dim_x, dim_y, dim_z) = (self.spatial_dim.x, self.spatial_dim.y, self.spatial_dim.z);
        let x = key.x.rem_euclid(dim_x);
        let y = key.y.rem_euclid(dim_y);
        let z = key.z.rem_euclid(dim_z);
        let index = ((x * dim_y + y) * dim_z + z) as usize;
        self.throttler.request(index)
    }

    pub(crate) fn set_now(&mut self, now: Instant) {
        self.throttler.set_now(now);
    }
}
