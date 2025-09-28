use crate::renderer::resources::shader::ShaderType;
use crate::renderer::resources::vx_buffer::VxBuffer;
use std::ops::Deref;
use wgpu::{BufferUsages, Device};

pub struct VxDevice {
    device: Device,
}

impl VxDevice {
    pub(crate) fn new(device: Device) -> Self {
        Self { device }
    }

    pub fn create_vx_buffer<T: ShaderType>(
        &self,
        label: &str,
        len: usize,
        buffer_usages: BufferUsages,
    ) -> VxBuffer
    {
        let shader_type_data = T::SHADER_TYPE_DATA;
        let stride = shader_type_data.stride;
        let name = shader_type_data.name;
        VxBuffer::new(
            &self.device,
            label,
            name,
            stride,
            len,
            buffer_usages,
        )
    }
}

impl Deref for VxDevice {
    type Target = Device;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}
