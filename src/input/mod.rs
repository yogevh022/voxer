pub mod keycode;

use winit::keyboard::KeyCode;

pub struct Input {
    kb_pressed: [bool; 255],
    kb_down: [bool; 255],
    kb_released: [bool; 255],
}

impl Default for Input {
    fn default() -> Self {
        Self {
            kb_pressed: [false; 255],
            kb_down: [false; 255],
            kb_released: [false; 255],
        }
    }
}

impl Input {
    pub(crate) fn press(&mut self, key_code: KeyCode) {
        let idx = keycode::keycode_index(key_code).expect("Invalid keycode");
        self.kb_pressed[idx as usize] = true;
        self.kb_down[idx as usize] = true;
    }

    pub(crate) fn release(&mut self, key_code: KeyCode) {
        let idx = keycode::keycode_index(key_code).expect("Invalid keycode");
        self.kb_pressed[idx as usize] = false;
        self.kb_down[idx as usize] = false;
        self.kb_released[idx as usize] = true;
    }

    pub fn keyboard_pressed(&self, key_code: KeyCode) -> bool {
        let idx = keycode::keycode_index(key_code).expect("Invalid keycode");
        self.kb_pressed[idx as usize]
    }

    pub fn keyboard_down(&self, key_code: KeyCode) -> bool {
        let idx = keycode::keycode_index(key_code).expect("Invalid keycode");
        self.kb_down[idx as usize]
    }

    pub fn keyboard_released(&self, key_code: KeyCode) -> bool {
        let idx = keycode::keycode_index(key_code).expect("Invalid keycode");
        self.kb_released[idx as usize]
    }
}
