use std::time::Instant;

pub struct Timer {
    last_frame: Instant,
    second_start: Instant,
    frames: u64,
    delta_time: f32,
    fps: f32,
}

impl Timer {
    pub(crate) fn tick(&mut self) {
        self.frames += 1;
        self.delta_time = self.last_frame.elapsed().as_secs_f32();
        self.last_frame = Instant::now();
        if self.second_start.elapsed().as_millis() > 500 { // bi-second updates
            self.fps = self.frames as f32 * 2f32;
            self.frames = 0;
            self.second_start = Instant::now();
        }
    }

    #[inline(always)]
    pub(crate) fn delta(&self) -> f32 {
        self.delta_time
    }

    #[inline(always)]
    pub(crate) fn fps(&self) -> f32 {
        self.fps
    }

    #[inline(always)]
    pub(crate) fn is_new_second(&self) -> bool {
        self.frames == 0
    }

    pub fn temp_200th_frame(&self) -> bool {
        self.frames % 200 == 0
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            last_frame: Instant::now(),
            second_start: Instant::now(),
            frames: 0,
            delta_time: 0.0,
            fps: 0.0,
        }
    }
}
