use crate::renderer::gpu::GPUChunkEntry;
use crate::renderer::gpu::chunk_manager::ChunkManager;
use crate::renderer::resources;
use crate::renderer::{Renderer, RendererBuilder};
use crate::world::types::Chunk;
use crate::{compute};
use glam::{IVec3, Mat4};
use std::sync::Arc;
use winit::window::Window;
use crate::vtypes::Camera;

pub struct AppRenderer<'window, const ChunkBuffers: usize, const ChunkStagingBuffers: usize> {
    pub renderer: Renderer<'window>,
    chunk_manager: ChunkManager<ChunkBuffers, ChunkStagingBuffers>,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl<const ChunkBuffers: usize, const ChunkStagingBuffers: usize>
    AppRenderer<'_, ChunkBuffers, ChunkStagingBuffers>
{
    pub fn load_chunks(&mut self, chunks: Vec<Chunk>) {
        self.chunk_manager.write_new(&self.renderer, chunks);
        // self.chunk_manager.malloc_debug();
    }

    pub fn unload_chunks(&mut self, positions: Vec<IVec3>) {
        for position in positions {
            self.chunk_manager.drop(position);
        }
    }

    pub fn is_chunk_rendered(&self, position: IVec3) -> bool {
        self.chunk_manager.is_rendered(position)
    }

    pub fn map_rendered_chunk_positions<F>(&self, func: F) -> Vec<IVec3>
    where
        F: FnMut(IVec3) -> bool,
    {
        self.chunk_manager.map_rendered_chunk_positions(func)
    }

    fn render_chunks(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        camera: &Camera,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.renderer.bind_groups.texture_atlas, &[]);
        render_pass.set_bind_group(1, &self.renderer.bind_groups.view_projection, &[]);

        let view_proj = camera.get_view_projection();
        self.renderer.write_view_projection(view_proj);

        self.chunk_manager.draw(&self.renderer, render_pass);
    }

    pub fn render(&mut self, camera: &Camera) -> Result<(), wgpu::SurfaceError> {
        let frame = self.renderer.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        // fixme not every frame..
        self.chunk_manager.poll_update_delta_draw();
        let mut encoder = self.renderer.create_encoder("render_encoder");
        {
            let mut render_pass = begin_render_pass(&mut encoder, &view, &self.renderer.depth_texture_view);
            self.render_chunks(&mut render_pass, camera);
        }

        self.renderer.queue.submit(Some(encoder.finish()));

        frame.present();
        Ok(())
    }
}

pub fn make_app_renderer<'a, const NumBuffers: usize, const NumStagingBuffers: usize>(
    window: Arc<Window>,
    render_distance: f32,
) -> AppRenderer<'a, NumBuffers, NumStagingBuffers> {
    let renderer_builder = RendererBuilder::new(window);

    let surface_format = renderer_builder.surface_format.unwrap();
    let renderer = renderer_builder.build();

    let render_pipeline = RendererBuilder::make_render_pipeline(
        &renderer.device,
        surface_format,
        resources::shader::main_shader().into(),
        &[
            &renderer.layouts.texture_atlas,   // 0
            &renderer.layouts.view_projection, // 1
            &renderer.layouts.mmat,            // 2
        ],
    );

    let max_buffer_size = compute::MIB * 128;
    let chunk_manager = ChunkManager::<NumBuffers, NumStagingBuffers>::new(
        &renderer,
        max_buffer_size,
        12_288 * size_of::<GPUChunkEntry>(), // fixme this is overkill
        12_288 * size_of::<Mat4>(),
    );

    AppRenderer {
        renderer,
        chunk_manager,
        render_pipeline,
    }
}

fn begin_render_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    frame_view: &wgpu::TextureView,
    depth_view: &wgpu::TextureView,
) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("render_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: frame_view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
    })
}
