pub mod keycode;
mod keyboard;
mod mouse;

#[derive(Default, Debug)]
pub struct Input {
    pub keyboard: keyboard::KeyboardInput,
    pub mouse: mouse::MouseInput,
}
