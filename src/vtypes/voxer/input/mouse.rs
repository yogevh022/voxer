#[derive(Debug)]
pub(crate) struct MouseInput {
    pub(crate) position: [f64; 2],
    pub(crate) delta: [f64; 2],
    pub(crate) pressed: [bool; 32],
    pub(crate) down: [bool; 32],
    pub(crate) released: [bool; 32],
}

impl Default for MouseInput {
    fn default() -> Self {
        Self {
            position: [0f64; 2],
            delta: [0f64; 2],
            pressed: [false; 32],
            down: [false; 32],
            released: [false; 32],
        }
    }
}

impl MouseInput {
    pub fn accumulate_delta(&mut self, delta: (f64, f64)) {
        self.delta = [self.delta[0] + delta.0, self.delta[1] + delta.1];
    }

    pub fn set_delta(&mut self, delta: (f64, f64)) {
        self.delta = [delta.0, delta.1];
    }

    pub fn set_position(&mut self, position: (f64, f64)) {
        self.position = [position.0, position.1];
    }
}
