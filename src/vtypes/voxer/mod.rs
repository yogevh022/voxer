pub mod input;
mod timer;
pub use input::Input;
pub use timer::VxTime;

use crate::vtypes::Camera;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct Voxer {
    pub time: VxTime,
    pub input: Arc<RwLock<Input>>,
    pub camera: Camera,
}

impl Default for Voxer {
    fn default() -> Self {
        Self {
            time: VxTime::default(),
            input: Arc::new(RwLock::new(Input::default())),
            camera: Camera::default(),
        }
    }
}
