use crate::compute::geo::{Frustum, Plane};
use crate::renderer::Renderer;
use crate::renderer::gpu::chunk_session::{GpuChunkSession, GpuChunkSessionConfig};
use crate::renderer::resources;
use crate::renderer::resources::shader::{MAX_WORKGROUP_DIM_1D, MAX_WORKGROUP_DIM_2D};
use crate::renderer::resources::texture::get_atlas_image;
use crate::renderer::resources::vx_buffer::VxBuffer;
use crate::vtypes::Camera;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, UVec2, Vec4};
use std::borrow::Cow;
use std::sync::Arc;
use voxer_macros::ShaderType;
use wgpu::{BindGroup, BufferUsages, CommandEncoder, RenderPipeline};
use winit::dpi::PhysicalSize;
use winit::window::Window;

#[repr(C, align(16))]
#[derive(ShaderType, Copy, Clone, Debug, Pod, Zeroable)]
pub struct UniformCameraView {
    // todo move from here
    view_proj: Mat4,
    view_planes: [Plane; 6],
    origin: Vec4, // w = voxel_render_distance
    view_dim_px: UVec2,
    fov_y: f32,
    _padding: u32,
}

impl UniformCameraView {
    pub fn new(camera: &Camera, voxel_render_distance: u32, window_size: PhysicalSize<u32>) -> Self {
        let view_proj = camera.view_projection();
        let view_planes = Frustum::planes(view_proj);
        let origin = camera
            .transform
            .position
            .extend(voxel_render_distance as f32);
        let view_dim_px = UVec2::new(window_size.width, window_size.height);
        Self {
            view_proj,
            view_planes,
            origin,
            view_dim_px,
            fov_y: camera.frustum.fov,
            _padding: 0,
        }
    }
}

pub struct AppRenderer<'window> {
    pub renderer: Renderer<'window>,
    pub chunk_session: GpuChunkSession, // fixme temp
    render_pipeline: RenderPipeline,
    atlas_bind_group: BindGroup,
    view_projection_buffer: VxBuffer,

    dbg_pipeline: RenderPipeline,
    dbg_bg: BindGroup,
}

impl AppRenderer<'_> {
    pub fn new(window: Arc<Window>) -> Self {
        let renderer = Renderer::new(window);

        let view_projection_buffer = renderer.device.create_vx_buffer::<UniformCameraView>(
            "View Projection Buffer",
            1,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        // fixme move to a config and fix arbitrary numbers
        let near_chunks = ((36.0 * 36.0 * 36.0) / 1.8) as usize;
        let cm_config = GpuChunkSessionConfig {
            max_chunks: near_chunks,
            max_write_count: 1 << 14, // arbitrary
            max_face_count: (near_chunks as f32 * 0.4f32) as usize * 4096, // rough temp fov + face estimate
            max_workgroup_size_1d: MAX_WORKGROUP_DIM_1D,
            max_workgroup_size_2d: MAX_WORKGROUP_DIM_2D,
            max_indirect_count: 1 << 16, // arbitrary
        };
        let chunk_session = GpuChunkSession::new(&renderer, &view_projection_buffer, cm_config);

        let v = &renderer.depth.mip_views[1];

        let (dbg_bgl, dbg_bg) = renderer.dbg_sampler(v);
        let dbg_pipeline = debug_make_render_pipeline(
            &renderer,
            resources::shader::dbg_render_wgsl().into(),
            &[
                &dbg_bgl,
            ],
        );

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

            dbg_pipeline,
            dbg_bg,
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
        camera: &Camera,
        voxel_render_distance: u32,
        window_size: PhysicalSize<u32>,
    ) -> Result<(), wgpu::SurfaceError> {
        let frame = self.renderer.surface.get_current_texture()?;
        let view = frame.texture.create_view(&Default::default());

        let camera_view = UniformCameraView::new(camera, voxel_render_distance, window_size);

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

            render_pass.set_pipeline(&self.dbg_pipeline);
            render_pass.set_bind_group(0, &self.dbg_bg, &[]);
            render_pass.draw(0..6, 0..1);
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

pub fn debug_make_render_pipeline(
    renderer: &Renderer<'_>,
    shader_source: Cow<str>,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> RenderPipeline {
    let shader = resources::shader::create_shader(
        &renderer.device,
        shader_source,
        "dbg render pipeline shader",
    );
    let dbg_render_pipeline_layout =
        &renderer
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("dbg_render_pipeline_layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

    renderer
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("dbg_render_pipeline"),
            layout: Some(&dbg_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("dbg_vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("dbg_fs_main"),
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
