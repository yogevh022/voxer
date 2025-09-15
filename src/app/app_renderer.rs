use crate::compute;
use crate::compute::geo::{Frustum, Plane};
use crate::renderer::Renderer;
use crate::renderer::gpu::GPUChunkEntry;
use crate::renderer::gpu::chunk_manager::ChunkManager;
use crate::renderer::resources;
use crate::renderer::resources::texture::get_atlas_image;
use crate::renderer::resources::vg_buffer_resource::VgBufferResource;
use crate::vtypes::Camera;
use crate::world::types::Chunk;
use bytemuck::{Pod, Zeroable};
use glam::IVec3;
use std::borrow::Cow;
use std::sync::Arc;
use wgpu::BindGroup;
use winit::window::Window;

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct UniformCameraView {
    view_proj: [f32; 16],
    frustum_planes: [Plane; 6],
}

pub struct AppRenderer<'window> {
    pub renderer: Renderer<'window>,
    chunk_manager: ChunkManager,
    render_pipeline: wgpu::RenderPipeline,
    atlas_bind_group: BindGroup,
    view_projection_buffer: VgBufferResource,
}

impl AppRenderer<'_> {
    pub fn load_chunks<'a>(&mut self, chunks: &mut impl Iterator<Item = &'a Chunk>) {
        self.chunk_manager.write_new(&self.renderer, chunks);
        // self.chunk_manager.malloc_debug();
    }

    pub fn unload_chunks(&mut self, positions: &Vec<IVec3>) {
        for &position in positions {
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

    fn render_chunks(&mut self, render_pass: &mut wgpu::RenderPass, camera: &Camera) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.atlas_bind_group, &[]);

        self.chunk_manager.draw(&self.renderer, render_pass);
    }

    pub fn render(&mut self, camera: &Camera) -> Result<(), wgpu::SurfaceError> {
        let frame = self.renderer.surface.get_current_texture()?;
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.renderer.create_encoder("render_encoder");

        let vp = camera.view_projection();
        let cam_view = UniformCameraView {
            view_proj: vp.to_cols_array(),
            frustum_planes: Frustum::planes(vp),
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

pub fn make_app_renderer<'a>(window: Arc<Window>) -> AppRenderer<'a> {
    let renderer = Renderer::new(window);

    let view_projection_buffer = VgBufferResource::new(
        &renderer.device,
        "View Projection Buffer",
        size_of::<UniformCameraView>(),
        wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    );

    let face_data_buffer_size = compute::MIB * 128;
    let chunk_manager = ChunkManager::new(
        &renderer,
        &view_projection_buffer,
        face_data_buffer_size as wgpu::BufferAddress,
        12_288 * size_of::<GPUChunkEntry>() as wgpu::BufferAddress, // fixme this is overkill
    );

    let (atlas_layout, atlas_bind_group) =
        renderer.texture_sampler("Texture Sampler Atlas", get_atlas_image());

    let render_pipeline = make_render_pipeline(
        &renderer,
        resources::shader::main_shader().into(),
        &[
            &atlas_layout,                           // 0
            &chunk_manager.render.bind_group_layout, // 1
        ],
    );

    AppRenderer {
        renderer,
        chunk_manager,
        render_pipeline,
        atlas_bind_group,
        view_projection_buffer,
    }
}

pub fn make_render_pipeline(
    renderer: &Renderer<'_>,
    shader_source: Cow<str>,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::RenderPipeline {
    let shader = resources::shader::create(&renderer.device, shader_source);
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
