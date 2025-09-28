use crate::renderer::resources::vx_buffer::VxBuffer;
use std::ops::Deref;
use wgpu::Queue;

pub struct VxQueue {
    queue: Queue,
    staging_buffer: VxBuffer,
}

impl VxQueue {
    pub(crate) fn new(queue: Queue, staging_buffer: VxBuffer) -> Self {
        Self {
            queue,
            staging_buffer,
        }
    }
}

impl Deref for VxQueue {
    type Target = Queue;

    fn deref(&self) -> &Self::Target {
        &self.queue
    }
}
