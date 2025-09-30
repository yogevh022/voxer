use std::time::{Duration, Instant};

pub struct Throttler {
    pub max_pending: usize,
    pub throttle_duration: Duration,
    now: Instant,
    queue: Box<[Instant]>,
}

impl Throttler {
    pub fn new(max_pending: usize, throttle_duration: Duration) -> Self {
        let now = Instant::now();
        let available_now = now - throttle_duration;
        Self {
            max_pending,
            throttle_duration,
            now,
            queue: vec![available_now; max_pending].into_boxed_slice(),
        }
    }

    pub fn set_now(&mut self, now: Instant) {
        self.now = now;
    }

    pub fn request(&mut self, key: usize) -> bool {
        debug_assert!(key < self.max_pending);
        if self.now.duration_since(self.queue[key]) >= self.throttle_duration {
            self.queue[key] = self.now;
            true
        } else {
            false
        }
    }
}
