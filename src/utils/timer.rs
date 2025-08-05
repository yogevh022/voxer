use std::time::Instant;

pub(crate) struct Timer {
    last_frame: Instant,
    second_start: Instant,
    frames: u64,
    pub delta_time: f32,
    pub fps: f32,
}

impl Timer {
    pub(crate) fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            second_start: Instant::now(),
            frames: 0,
            delta_time: 0.0,
            fps: 0.0,
        }
    }

    pub(crate) fn tick(&mut self) {
        self.frames += 1;
        self.delta_time = self.last_frame.elapsed().as_secs_f32();
        self.last_frame = Instant::now();
        if self.second_start.elapsed().as_secs() > 1 {
            self.fps = self.frames as f32;
            self.frames = 0;
            self.second_start = Instant::now();
        }
    }
}
