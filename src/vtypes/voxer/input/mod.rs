mod keyboard;
mod keycode;
mod mouse;
pub use keycode::get_keycode;

#[derive(Default, Debug)]
pub struct Input {
    pub keyboard: keyboard::KeyboardInput,
    pub mouse: mouse::MouseInput,
}
