use crate::render::renderer::resources::{
    MeshBuffers, RenderResources, TerrainResources, UniformResources,
};
use crate::render::types::Vertex;
use crate::render::{bind_group, index, pipeline, shader, texture, uniform, vertex};
use crate::utils;
pub(crate) use chunks::ChunkBufferEntry;
use std::sync::Arc;
use winit::window::Window;

mod chunks;
mod render;

pub(crate) struct Renderer<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    pipeline: wgpu::RenderPipeline,
    resources: RenderResources,
}

impl Renderer<'_> {
    pub(crate) async fn new(window: Arc<Window>) -> Self {
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
                required_features: wgpu::Features::empty(),
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
        let atlas_texture = texture::create_diffuse(&device, &queue, &atlas_rgba);
        let atlas_texture_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let atlas_sampler = texture::diffuse_sampler(&device);
        let terrain_texture_bg_entries = bind_group::index_based_entries([
            // the index of the resource in this array is the index of the binding
            wgpu::BindingResource::TextureView(&atlas_texture_view),
            wgpu::BindingResource::Sampler(&atlas_sampler),
        ]);

        let uniform_buffer = uniform::create_buffer(
            &device,
            // arbitrarily creating the buffer with capacity for 1000 objects
            (device.limits().min_uniform_buffer_offset_alignment * 1000) as u64,
        );

        let uniform_buffer_binding_resource = wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &uniform_buffer,
            offset: 0,
            size: std::num::NonZeroU64::new(64),
        });

        let (tex_bg_layout, tex_bg) =
            texture::create_bind_group(&device, &terrain_texture_bg_entries);
        let (uniform_bg_layout, uniform_bg) =
            uniform::create_bind_group(&device, uniform_buffer_binding_resource);

        let shader = shader::create(&device, shader::main_shader_source().into());
        let pipeline = pipeline::create(
            &device,
            &[&tex_bg_layout, &uniform_bg_layout],
            &shader,
            surface_format,
        );

        let terrain_vertex_alloc = 1024 * size_of::<Vertex>() as u64;
        let terrain_index_alloc = 1024 * size_of::<u32>() as u64;
        // let (terrain_vertex_alloc, terrain_index_alloc) =
        //     alloc::size_of_meshes_from_sos(rendered_objects);

        let terrain_resources = TerrainResources {
            mesh_buffers: MeshBuffers {
                vertex: vertex::create_buffer(&device, terrain_vertex_alloc),
                index: index::create_buffer(&device, terrain_index_alloc),
            },
            texture_view: atlas_texture_view,
            texture_sampler: atlas_sampler,
            texture_bind_group: tex_bg,
        };

        let uniform_resources = UniformResources {
            buffer: uniform_buffer,
            bind_group: uniform_bg,
        };

        let depth_texture = texture::create_depth(&device, &config);
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let resources = RenderResources {
            terrain: terrain_resources,
            uniform: uniform_resources,
            depth_texture_view,
            chunk_buffer_pool: Vec::new(),
        };

        Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            resources,
        }
    }
}
