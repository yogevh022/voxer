use std::time::Instant;

pub struct VxTime {
    frame: usize,
    delta_time: f32,
    delta_time_history: Box<[f32]>,
    delta_time_history_sum: f32,
    last_frame_time: Instant,
}

impl VxTime {
    pub fn new(frame_history: usize) -> Self {
        debug_assert!(frame_history > 0);
        Self {
            frame: 0,
            delta_time: 0.0,
            delta_time_history: vec![0.0; frame_history].into_boxed_slice(),
            delta_time_history_sum: 0.0,
            last_frame_time: Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        let this_frame = Instant::now();
        self.delta_time = this_frame
            .duration_since(self.last_frame_time)
            .as_secs_f32();
        self.last_frame_time = this_frame;
        self.frame += 1;

        unsafe {
            let frame_idx = self.frame % self.delta_time_history.len();
            let history_slot = self.delta_time_history.get_unchecked_mut(frame_idx);
            self.delta_time_history_sum += self.delta_time - *history_slot;
            *history_slot = self.delta_time;
        }
    }

    #[inline]
    pub fn fps_avg(&self) -> f32 {
        self.delta_time_history.len() as f32 / self.delta_time_history_sum
    }

    #[inline]
    pub fn fps_rt(&self) -> f32 {
        1.0 / self.delta_time
    }

    #[inline]
    pub fn dt(&self) -> f32 {
        self.delta_time
    }

    #[inline]
    pub fn frame(&self) -> usize {
        self.frame
    }
}

impl Default for VxTime {
    fn default() -> Self {
        Self::new(240)
    }
}