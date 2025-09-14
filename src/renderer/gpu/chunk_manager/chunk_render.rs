use crate::renderer::gpu::chunk_manager::{BufferDrawArgs, MultiDrawInstruction};
use crate::renderer::{DrawIndexedIndirectArgsA32, Renderer, RendererBuilder, resources};

pub struct ChunkRender {
    pub face_data_buffer: wgpu::Buffer,
    pub chunk_translations_buffer: wgpu::Buffer,
    face_data_bind_group: wgpu::BindGroup,
    chunk_translations_bind_group: wgpu::BindGroup,
}

impl ChunkRender {
    pub fn init(
        renderer: &Renderer<'_>,
        face_data_buffer_size: wgpu::BufferAddress,
        chunk_translations_buffer_size: wgpu::BufferAddress,
        fd_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let face_data_buffer = RendererBuilder::make_buffer(
            &renderer.device,
            "face_data_buffer",
            face_data_buffer_size,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
        );
        let face_data_bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("face_data_bind_group"),
                layout: fd_layout,
                entries: &resources::utils::index_based_entries([
                    face_data_buffer.as_entire_binding(), // 0
                ]),
            });
        let chunk_translations_buffer = RendererBuilder::make_buffer(
            &renderer.device,
            "chunk_translations_buffer",
            chunk_translations_buffer_size,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
        );
        let chunk_translations_bind_group =
            renderer
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("chunk_translations_bind_group"),
                    layout: &renderer.layouts.mmat, // fixme rename, even move away from renderer
                    entries: &resources::utils::index_based_entries([
                        chunk_translations_buffer.as_entire_binding(), // 0
                    ]),
                });

        Self {
            face_data_buffer,
            chunk_translations_buffer,
            face_data_bind_group,
            chunk_translations_bind_group,
        }
    }

    pub fn write_args_to_indirect_buffer(
        &self,
        renderer: &Renderer<'_>,
        buffer_draw_args: &BufferDrawArgs,
    ) {
        let flat_draw_args = buffer_draw_args.values().cloned().collect::<Vec<_>>();
        renderer.write_buffer(
            &renderer.indirect_buffer,
            0,
            bytemuck::cast_slice(&flat_draw_args),
        );
    }

    pub fn draw(&self, renderer: &Renderer<'_>, render_pass: &mut wgpu::RenderPass, count: u32) {
        render_pass.set_bind_group(2, &self.chunk_translations_bind_group, &[]);
        render_pass.set_bind_group(3, &self.face_data_bind_group, &[]);
        if count != 0 {
            render_pass.multi_draw_indirect(&renderer.indirect_buffer, 0, count);
        }
    }
}
