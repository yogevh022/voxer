use super::RendererBuilder;

pub struct RendererBindGroupLayouts {
    pub mmat: wgpu::BindGroupLayout,
    pub view_projection: wgpu::BindGroupLayout,
    pub texture_atlas: wgpu::BindGroupLayout,
}

pub struct RendererBindGroups {
    pub view_projection: wgpu::BindGroup,
    pub texture_atlas: wgpu::BindGroup,
}

pub(crate) struct Renderer<'window> {
    pub(crate) surface: wgpu::Surface<'window>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) indirect_buffer: wgpu::Buffer,
    pub(crate) depth_texture_view: wgpu::TextureView,
    pub(crate) layouts: RendererBindGroupLayouts,
    pub(crate) bind_groups: RendererBindGroups,
    pub(crate) view_projection_buffer: wgpu::Buffer,
}

impl<'window> From<RendererBuilder<'window>> for Renderer<'window> {
    fn from(value: RendererBuilder<'window>) -> Self {
        let layouts = RendererBindGroupLayouts {
            mmat: value.mmat_layout.unwrap(),
            view_projection: value.view_projection_layout.unwrap(),
            texture_atlas: value.texture_atlas_layout.unwrap(),
        };
        let bind_groups = RendererBindGroups {
            view_projection: value.view_projection_bind_group.unwrap(),
            texture_atlas: value.texture_atlas_bind_group.unwrap(),
        };
        Self {
            surface: value.surface.unwrap(),
            device: value.device.unwrap(),
            queue: value.queue.unwrap(),
            indirect_buffer: value.indirect_buffer.unwrap(),
            depth_texture_view: value.depth_texture_view.unwrap(),
            view_projection_buffer: value.view_projection_buffer.unwrap(),
            layouts,
            bind_groups,
        }
    }
}

impl Renderer<'_> {
    pub fn write_buffer(&self, buffer: &wgpu::Buffer, offset: wgpu::BufferAddress, data: &[u8]) {
        self.queue.write_buffer(buffer, offset, data)
    }

    pub fn create_encoder(&self, label: &str) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) })
    }
}
