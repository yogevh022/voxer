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

    pub chunk_manager: ChunkManager<ChunkBuffers, ChunkStagingBuffers>,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl<const ChunkBuffers: usize, const ChunkStagingBuffers: usize>
    AppRenderer<'_, ChunkBuffers, ChunkStagingBuffers>
{
    pub fn write_new_chunks(&mut self, chunks: Vec<(usize, Chunk)>) {
        self.chunk_manager.write_new(&self.renderer, chunks);
    }

    pub fn update_current_draw(&mut self) {
        // let mut delta_lock = self.draw_delta.write();
        // if delta_lock.changed {
        //     for i in 0..BUFF_N {
        //         self.current_draw[i].extend(delta_lock.args[i].drain());
        //     }
        //     delta_lock.changed = false;
        // }
    }

    pub fn unload_chunks(&mut self, chunks: Vec<IVec3>) {
        // for chunk_pos in chunks {
        //     let chunk_alloc = self
        //         .chunk_position_to_allocation
        //         .remove(&chunk_pos)
        //         .unwrap();
        //     self.current_draw[chunk_alloc.0].remove(&chunk_alloc.1.vertex_offset);
        //     if let Err(e) = self.chunk_malloc.free(chunk_alloc) {
        //         // todo no need to check here
        //         println!("failed to free chunk: {:?}, {:?}", chunk_pos, chunk_alloc);
        //     }
        // }
    }

    pub fn encode_render_pass(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        camera: &Camera,
    ) {
        
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.renderer.bind_groups.texture_atlas, &[]);
        render_pass.set_bind_group(1, &self.renderer.bind_groups.view_projection, &[]);

        let view_proj = camera.get_view_projection();
        self.renderer.write_view_projection(view_proj);

        // let multi_draw_instructions = self
        //     .chunk_render
        //     .write_commands_to_indirect_buffer(&self.renderer, &self.current_draw);
        //
        // self.chunk_render
        //     .multi_draw(&self.renderer, &mut render_pass, multi_draw_instructions);
    }

    pub fn render(&mut self, camera: &Camera) -> Result<(), wgpu::SurfaceError> {
        let frame = self.renderer.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.renderer.create_encoder("render_encoder");
        {
            let mut render_pass = begin_render_pass(&mut encoder, &view, &self.renderer.depth_texture_view);
            self.encode_render_pass(&mut render_pass, camera);
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

    let max_rendered_chunks = compute::geo::max_discrete_sphere_pts(render_distance);
    let max_buffer_size = compute::MIB * 128;
    let chunk_manager = ChunkManager::<NumBuffers, NumStagingBuffers>::new(
        &renderer,
        max_buffer_size,
        max_rendered_chunks * size_of::<GPUChunkEntry>(), // fixme this is overkill
        max_rendered_chunks * size_of::<Mat4>(),
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
