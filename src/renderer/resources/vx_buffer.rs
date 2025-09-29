use crate::renderer::resources::shader::VxShaderTypeData;
use std::ops::Deref;
use wgpu::{
    BindGroupLayoutEntry, BufferAddress, BufferBindingType, BufferSize, BufferUsages, ShaderStages,
};

pub struct VxBuffer {
    buffer: wgpu::Buffer,
    stride: usize,
    shader_type: &'static str,
}

impl VxBuffer {
    pub(crate) fn new(
        device: &wgpu::Device,
        label: &str,
        shader_type_name: &'static str,
        stride: usize,
        len: usize,
        buffer_usages: BufferUsages,
    ) -> Self {
        let buffer_size = match buffer_usages {
            BufferUsages::UNIFORM => std::cmp::max(
                len * stride,
                device.limits().min_uniform_buffer_offset_alignment as usize,
            ),
            _ => len * stride,
        };
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: buffer_size as BufferAddress,
            usage: buffer_usages,
            mapped_at_creation: false,
        });
        Self {
            buffer,
            stride,
            shader_type: shader_type_name,
        }
    }

    pub fn stride(&self) -> usize {
        self.stride
    }
    
    pub fn bind_layout_entry(
        &self,
        binding: u32,
        read_only: bool,
        visibility: ShaderStages,
    ) -> BindGroupLayoutEntry {
        let buffer_binding_type = match self.buffer.usage() {
            usage if usage.contains(BufferUsages::STORAGE) => {
                BufferBindingType::Storage { read_only }
            }
            usage if usage.contains(BufferUsages::UNIFORM) => BufferBindingType::Uniform,
            _ => panic!("Unsupported buffer usage"),
        };
        let binding_type = wgpu::BindingType::Buffer {
            ty: buffer_binding_type,
            has_dynamic_offset: false,
            min_binding_size: BufferSize::new(self.buffer.size()),
        };
        BindGroupLayoutEntry {
            binding,
            visibility,
            ty: binding_type,
            count: None,
        }
    }
}

impl Deref for VxBuffer {
    type Target = wgpu::Buffer;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
