use std::ops::Deref;
use wgpu::Queue;

pub struct VxQueue {
    queue: Queue,
}

impl VxQueue {
    pub(crate) fn new(queue: Queue) -> Self {
        Self {
            queue,
        }
    }
}

impl Deref for VxQueue {
    type Target = Queue;

    fn deref(&self) -> &Self::Target {
        &self.queue
    }
}
