use crate::render::renderer;
use crate::render::renderer::resources::{
    ChunkPool, RenderResources, TerrainResources, TransformResources,
};
use crate::utils;
use std::sync::Arc;
use winit::window::Window;

mod chunks;
mod render;

pub(crate) struct Renderer<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    render_pipeline: wgpu::RenderPipeline,
    render_resources: RenderResources,
}

impl Renderer<'_> {
    pub(crate) async fn new(window: Arc<Window>, render_distance: f32) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::VERTEX_WRITABLE_STORAGE,
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
                trace: Default::default(),
            })
            .await
            .unwrap();

        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let atlas_rgba = utils::temp::get_atlas_image();
        let atlas_texture =
            renderer::helpers::texture::create_diffuse(&device, &queue, &atlas_rgba);
        let atlas_texture_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let atlas_sampler = renderer::helpers::texture::diffuse_sampler(&device);
        let terrain_texture_bg_entries = renderer::helpers::bind_group::index_based_entries([
            wgpu::BindingResource::TextureView(&atlas_texture_view),
            wgpu::BindingResource::Sampler(&atlas_sampler),
        ]);

        let uniform_buffer_size = device
            .limits()
            .min_uniform_buffer_offset_alignment
            .min(size_of::<[[f32; 4]; 4]>() as u32) as u64;
        let uniform_buffer = renderer::helpers::uniform::create_buffer(
            &device,
            uniform_buffer_size,
            "uniform_buffer",
        );

        let view_projection_buffer = wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &uniform_buffer,
            offset: 0,
            size: std::num::NonZeroU64::new(uniform_buffer_size),
        });

        // >upper bound of max chunks to be buffered at once
        let max_rendered_chunks = utils::geo::max_discrete_sphere_pts(render_distance);
        let chunk_matrix_buffer =
            renderer::helpers::chunk_model::create_buffer(&device, max_rendered_chunks);

        let (texture_layout, atlas_bind_group) =
            renderer::helpers::texture::create_bind_group(&device, &terrain_texture_bg_entries);
        let (transform_matrices_layout, transform_matrices_bind_group) =
            renderer::helpers::transform_matrices::create_bind_group(
                &device,
                &renderer::helpers::bind_group::index_based_entries([
                    view_projection_buffer,                  // 0
                    chunk_matrix_buffer.as_entire_binding(), // 1
                ]),
            );

        let render_pipeline = renderer::helpers::render_pipeline::create(
            &device,
            renderer::helpers::shader::main_shader_source().into(),
            &[&texture_layout, &transform_matrices_layout],
            surface_format,
            "main_render_pipeline",
        );

        let terrain_resources = TerrainResources {
            atlas_view: atlas_texture_view,
            atlas_sampler,
            atlas_bind_group,
        };

        let transform_resources = TransformResources {
            uniform_buffer,
            model_matrix_buffer: chunk_matrix_buffer,
            bind_group: transform_matrices_bind_group,
        };

        let depth_texture_view = renderer::helpers::texture::create_depth(&device, &config)
            .create_view(&wgpu::TextureViewDescriptor::default());

        let render_resources = RenderResources {
            terrain: terrain_resources,
            transform: transform_resources,
            chunk_pool: ChunkPool::default(),
            depth_texture_view,
        };

        Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            render_resources,
        }
    }
}
