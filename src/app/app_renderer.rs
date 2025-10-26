use crate::renderer::Renderer;
use crate::renderer::gpu::chunk_session::{GpuChunkSession, GpuChunkSessionConfig};
use crate::renderer::resources;
use crate::renderer::resources::shader::{MAX_WORKGROUP_DIM_1D, MAX_WORKGROUP_DIM_2D};
use crate::renderer::resources::texture::get_atlas_image;
use crate::renderer::resources::vx_buffer::VxBuffer;
use crate::vtypes::Camera;
use std::borrow::Cow;
use std::sync::Arc;
use wgpu::{BindGroup, BufferUsages, CommandEncoder, RenderPipeline};
use winit::dpi::PhysicalSize;
use winit::window::Window;
use crate::renderer::gpu::vx_gpu_camera::VxGPUCamera;

pub struct AppRenderer<'window> {
    pub renderer: Renderer<'window>,
    pub chunk_session: GpuChunkSession, // fixme temp
    render_pipeline: RenderPipeline,
    atlas_bind_group: BindGroup,
    view_projection_buffer: VxBuffer,
}

impl AppRenderer<'_> {
    pub fn new(window: Arc<Window>) -> Self {
        let renderer = Renderer::new(window);

        let view_projection_buffer = renderer.device.create_vx_buffer::<VxGPUCamera>(
            "View Projection Buffer",
            1,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        // fixme move to a config and fix arbitrary numbers
        let near_chunks = ((48.0 * 48.0 * 48.0) * 0.55) as usize;
        let cm_config = GpuChunkSessionConfig {
            max_chunks: near_chunks,
            max_write_count: 1 << 14, // arbitrary
            max_face_count: (near_chunks as f32 * 0.4f32) as usize * 4096, // rough temp fov + face estimate
            max_workgroup_size_1d: MAX_WORKGROUP_DIM_1D,
            max_workgroup_size_2d: MAX_WORKGROUP_DIM_2D,
            max_indirect_count: 1 << 16, // arbitrary
        };
        let chunk_session = GpuChunkSession::new(&renderer, &view_projection_buffer, cm_config);

        let (atlas_layout, atlas_bind_group) =
            renderer.texture_sampler("Texture Sampler Atlas", get_atlas_image());

        let render_pipeline = make_render_pipeline(
            &renderer,
            resources::shader::render_wgsl().into(),
            &[
                &atlas_layout,               // 0
                &chunk_session.render_bgl(), // 1
            ],
        );

        Self {
            renderer,
            chunk_session,
            render_pipeline,
            atlas_bind_group,
            view_projection_buffer,
        }
    }

    fn render_chunks(&mut self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.atlas_bind_group, &[]);
        self.chunk_session
            .render_chunks(&self.renderer, render_pass);
    }

    pub fn submit_render_pass(
        &mut self,
        mut encoder: CommandEncoder,
        main_camera: &Camera,
        voxel_culling_distance: u32,
        window_size: PhysicalSize<u32>,
    ) -> Result<(), wgpu::SurfaceError> {
        let frame = self.renderer.surface.get_current_texture()?;
        let view = frame.texture.create_view(&Default::default());

        let camera_view = VxGPUCamera::new(main_camera, voxel_culling_distance, window_size);

        self.renderer.write_buffer(
            &self.view_projection_buffer,
            0,
            bytemuck::bytes_of(&camera_view),
        );

        {
            let mut render_pass =
                self.renderer
                    .begin_render_pass(&mut encoder, "Main Render Pass", &view);
            self.render_chunks(&mut render_pass);
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
    let shader =
        resources::shader::create_shader(&renderer.device, shader_source, "render pipeline shader");
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
            label: Some("Render Pipeline"),
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
