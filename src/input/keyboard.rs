use winit::keyboard::KeyCode;
use crate::input::keycode;

#[derive(Debug)]
pub(crate) struct KeyboardInput {
    pub pressed: [bool; 255],
    pub down: [bool; 255],
    pub released: [bool; 255],
}

impl Default for KeyboardInput {
    fn default() -> Self {
        Self {
            pressed: [false; 255],
            down: [false; 255],
            released: [false; 255],
        }
    }
}

impl KeyboardInput {
    pub(crate) fn press(&mut self, key_code: KeyCode) {
        let idx = keycode::keycode_index(key_code).expect("Invalid keycode");
        self.pressed[idx as usize] = true;
        self.down[idx as usize] = true;
    }

    pub(crate) fn release(&mut self, key_code: KeyCode) {
        let idx = keycode::keycode_index(key_code).expect("Invalid keycode");
        self.pressed[idx as usize] = false;
        self.down[idx as usize] = false;
        self.released[idx as usize] = true;
    }

    pub fn key_pressed(&self, key_code: KeyCode) -> bool {
        let idx = keycode::keycode_index(key_code).expect("Invalid keycode");
        self.pressed[idx as usize]
    }

    pub fn key_down(&self, key_code: KeyCode) -> bool {
        let idx = keycode::keycode_index(key_code).expect("Invalid keycode");
        self.down[idx as usize]
    }

    pub fn key_released(&self, key_code: KeyCode) -> bool {
        let idx = keycode::keycode_index(key_code).expect("Invalid keycode");
        self.released[idx as usize]
    }
}
