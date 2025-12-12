use glam::IVec3;
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
            unsafe {
                *self.queue.get_unchecked_mut(key) = self.now;
            };
        }
        can_request
    }

    fn set_now(&mut self, now: Instant) {
        self.now = now;
    }
}

pub struct PositionThrottler {
    throttler: BaseThrottler,
}

impl Throttler for PositionThrottler {
    type Key = IVec3;

    fn new(max_pending: usize, throttle_duration: Duration) -> Self {
        Self {
            throttler: BaseThrottler::new(max_pending, throttle_duration),
        }
    }

    fn request(&mut self, key: Self::Key) -> bool {
        let throttle_idx = smallhash::u32x3_to_18_bits(key.to_array());
        self.throttler.request(throttle_idx as usize)
    }

    fn set_now(&mut self, now: Instant) {
        self.throttler.set_now(now);
    }
}
