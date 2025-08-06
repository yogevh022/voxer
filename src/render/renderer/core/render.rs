use crate::render::render_pass;
use crate::render::renderer::core::Renderer;
use crate::{types, utils};
use glam::Mat4;

impl Renderer<'_> {
    pub(crate) fn render(
        &mut self,
        camera: &types::Camera,
        scene: &mut types::Scene,
    ) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });

        let mut render_pass =
            render_pass::begin(&mut encoder, &view, &self.resources.depth_texture_view);
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.resources.terrain.texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.resources.terrain.mesh_buffers.vertex.slice(..));
        render_pass.set_index_buffer(
            self.resources.terrain.mesh_buffers.index.slice(..),
            wgpu::IndexFormat::Uint32,
        );

        let uni_buf_offset = self.device.limits().min_uniform_buffer_offset_alignment;
        let vp = camera.get_view_projection();
        for (i, chunk_buffer_entry) in self.resources.chunk_buffer_pool.iter().enumerate() {
            let w_pos = utils::world::chunk_to_world_pos(&chunk_buffer_entry.position);
            let uni = (vp * Mat4::from_translation(w_pos)).to_cols_array_2d();

            self.queue.write_buffer(
                &self.resources.uniform.buffer,
                i as u64 * uni_buf_offset as u64,
                bytemuck::cast_slice(&[uni]),
            );
        }

        for (i, chunk_buffer_entry) in self.resources.chunk_buffer_pool.iter().enumerate() {
            let buf_offset = i as u32 * uni_buf_offset;
            let idx = chunk_buffer_entry.index_offset;
            render_pass.set_bind_group(1, &self.resources.uniform.bind_group, &[buf_offset]);
            render_pass.set_vertex_buffer(
                0,
                self.resources.chunk_buffer_pool[i].buffer.vertex.slice(..),
            );
            render_pass.set_index_buffer(
                self.resources.chunk_buffer_pool[i].buffer.index.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed(0..idx, 0, 0..1);
        }
        drop(render_pass);

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }
}
