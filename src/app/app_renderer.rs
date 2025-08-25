use crate::renderer::gpu::chunk_manager::ChunkManager;
use crate::renderer::gpu::{
    GPUChunkEntry, MeshVMallocMultiBuffer, MultiBufferMeshAllocation, VirtualMalloc,
};
use crate::renderer::resources;
use crate::renderer::{Index, Renderer, RendererBuilder, Vertex};
use crate::world::types::Chunk;
use crate::{call_every, compute, vtypes};
use glam::{IVec3, Mat4};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::{array, mem};
use wgpu::util::DrawIndexedIndirectArgs;
use winit::window::Window;

#[derive(Debug, Clone)]
pub struct DrawDelta<const N: usize> {
    pub changed: bool,
    pub args: [HashMap<u32, DrawIndexedIndirectArgs>; N],
}

impl<const N: usize> Default for DrawDelta<N> {
    fn default() -> Self {
        Self {
            changed: false,
            args: array::from_fn(|_| HashMap::new()),
        }
    }
}

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
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        camera: &vtypes::Camera,
    ) {
        let mut render_pass =
            resources::render_pass::begin(encoder, view, &self.renderer.depth_texture_view);
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.renderer.bind_groups.texture_atlas, &[]);
        render_pass.set_bind_group(1, &self.renderer.bind_groups.view_projection, &[]);

        let view_proj = camera.get_view_projection().to_cols_array();
        self.renderer.write_buffer(
            &self.renderer.view_projection_buffer,
            0u64,
            bytemuck::cast_slice(&[view_proj]),
        );

        // let multi_draw_instructions = self
        //     .chunk_render
        //     .write_commands_to_indirect_buffer(&self.renderer, &self.current_draw);
        // 
        // self.chunk_render
        //     .multi_draw(&self.renderer, &mut render_pass, multi_draw_instructions);
    }

    pub fn render(&mut self, camera: &vtypes::Camera) -> Result<(), wgpu::SurfaceError> {
        let frame = self.renderer.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.update_current_draw(); // fixme not here..

        let mut encoder =
            self.renderer
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("render_encoder"),
                });

        self.encode_render_pass(&mut encoder, &view, camera);

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
    let max_buffer_size = (compute::MIB * 128);
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

pub fn get_atlas_image() -> image::RgbaImage {
    let q = std::env::current_exe().unwrap();
    let project_root = q
        .parent() // target/debug
        .unwrap()
        .parent() // target
        .unwrap()
        .parent() // project root
        .unwrap();

    let atlas_image = image::open(project_root.join("src/renderer/texture/images/atlas.png"))
        .expect("failed to load atlas.png");
    atlas_image.to_rgba8()
}
