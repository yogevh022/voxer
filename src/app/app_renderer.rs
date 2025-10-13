use crate::renderer::Renderer;
use crate::renderer::gpu::chunk_manager::ChunkManager;
use crate::renderer::resources;
use crate::renderer::resources::texture::get_atlas_image;
use crate::renderer::resources::vx_buffer::VxBuffer;
use crate::vtypes::Camera;
use crate::world::types::Chunk;
use bytemuck::{Pod, Zeroable};
use glam::{IVec3, Mat4};
use std::borrow::Cow;
use std::sync::Arc;
use voxer_macros::ShaderType;
use wgpu::{BindGroup, BufferUsages, CommandEncoder, ComputePass, RenderPipeline};
use winit::window::Window;
use crate::compute::geo::{Frustum, Plane};

#[repr(C, align(16))]
#[derive(ShaderType, Copy, Clone, Debug, Pod, Zeroable)]
pub struct UniformCameraView {
    // todo move from here
    view_proj: Mat4,
    view_planes: [Plane; 6],
}

pub struct AppRenderer<'window> {
    pub renderer: Renderer<'window>,
    pub chunk_manager: ChunkManager, // fixme temp
    render_pipeline: RenderPipeline,
    atlas_bind_group: BindGroup,
    view_projection_buffer: VxBuffer,
}

impl AppRenderer<'_> {
    pub fn new(window: Arc<Window>, rdist: i32) -> Self {
        let renderer = Renderer::new(window);

        let view_projection_buffer = renderer.device.create_vx_buffer::<UniformCameraView>(
            "View Projection Buffer",
            1,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let max_chunk_write_count = 1 << 14; // fixme move to a config how many chunks can be written at once
        let max_chunk_count = 24 * 24 * 24; // fixme arbitrary number, move to a config
        let max_face_count = max_chunk_count * 4096; // arbitrary...
        let chunk_manager = ChunkManager::new(
            &renderer,
            rdist,
            &view_projection_buffer,
            max_face_count,
            max_chunk_count,
            max_chunk_write_count,
        );

        let (atlas_layout, atlas_bind_group) =
            renderer.texture_sampler("Texture Sampler Atlas", get_atlas_image());

        let render_pipeline = make_render_pipeline(
            &renderer,
            resources::shader::main_shader().into(),
            &[
                &atlas_layout,                           // 0
                &chunk_manager.render_bind_group_layout, // 1
            ],
        );

        Self {
            renderer,
            chunk_manager,
            render_pipeline,
            atlas_bind_group,
            view_projection_buffer,
        }
    }
    // pub fn update_new_chunks(&mut self, chunks: &[Chunk]) {
    //     self.chunk_manager.update_gpu_chunk_writes(chunks);
    // }
    //
    // pub fn encode_new_chunks(&mut self, compute_pass: &mut ComputePass) {
    //     self.chunk_manager.encode_gpu_chunk_writes(&self.renderer, compute_pass);
    // }

    // pub fn update_view_chunks(&mut self, view_planes: &[Plane; 6] ) {
    //     self.chunk_manager.update_gpu_view_chunks(view_planes);
    // }

    // pub fn encode_view_chunks(&mut self, compute_pass: &mut ComputePass) {
    //     self.chunk_manager.encode_gpu_view_chunks(&self.renderer, compute_pass);
    // }

    pub fn is_chunk_cached(&self, position: IVec3) -> bool {
        self.chunk_manager.is_chunk_cached(&position)
    }

    // pub fn retain_chunk_positions<F: FnMut(&IVec3) -> bool>(&mut self, func: F) {
    //     self.chunk_manager.retain_chunk_positions(func);
    // }

    fn render_chunks(&mut self, render_pass: &mut wgpu::RenderPass, camera: &Camera) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.atlas_bind_group, &[]);

        self.chunk_manager.draw(&self.renderer, render_pass);
    }

    pub fn submit_render_pass(
        &mut self,
        mut encoder: CommandEncoder,
        camera: &Camera,
    ) -> Result<(), wgpu::SurfaceError> {
        let frame = self.renderer.surface.get_current_texture()?;
        let view = frame.texture.create_view(&Default::default());

        let cam_view = UniformCameraView {
            view_proj: camera.view_projection(),
            view_planes: Frustum::planes(camera.view_projection()), // fixme perf
        };

        self.renderer.write_buffer(
            &self.view_projection_buffer,
            0,
            bytemuck::bytes_of(&cam_view),
        );

        {
            let mut render_pass =
                self.renderer
                    .begin_render_pass(&mut encoder, "Main Render Pass", &view);
            self.render_chunks(&mut render_pass, camera);
        }

        self.renderer.queue.submit(Some(encoder.finish()));

        frame.present();
        Ok(())
    }
}

pub fn make_render_pipeline(
    renderer: &Renderer<'_>,
    shader_source: Cow<str>,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> RenderPipeline {
    let shader = resources::shader::create(&renderer.device, shader_source, "render pipeline shader");
    let render_pipeline_layout =
        &renderer
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

    renderer
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: renderer.surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        })
}
